use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, Write};
use std::path::Path;

use crate::error::{BondError, Result};

#[allow(dead_code)]
pub struct FileOps;

impl FileOps {
    /// Read entire file contents
    #[allow(dead_code)]
    pub fn read_file(path: &str) -> Result<String> {
        fs::read_to_string(path).map_err(BondError::WriteConfigError)
    }

    /// Write contents to file atomically (via temp file)
    #[allow(dead_code)]
    pub fn write_file_atomically(path: &str, contents: &str) -> Result<()> {
        let path_obj = Path::new(path);
        let parent = path_obj.parent().unwrap_or(Path::new(""));

        // Create parent directory if needed
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }

        let temp_path = path_obj.with_extension("tmp");

        // Write to temp file
        let mut file = File::create(&temp_path)?;
        file.write_all(contents.as_bytes())?;
        file.sync_all()?;

        // Rename to target (atomic on POSIX)
        fs::rename(temp_path, path_obj)?;

        Ok(())
    }

    /// Append contents to file
    #[allow(dead_code)]
    pub fn append_file(path: &str, contents: &str) -> Result<()> {
        let mut file = OpenOptions::new().create(true).append(true).open(path)?;
        file.write_all(contents.as_bytes())?;
        Ok(())
    }

    /// Check if path exists
    #[allow(dead_code)]
    pub fn exists(path: &str) -> bool {
        Path::new(path).exists()
    }

    /// Create directory if not exists
    #[allow(dead_code)]
    pub fn ensure_dir(path: &str) -> Result<()> {
        if !Path::new(path).exists() {
            fs::create_dir_all(path)?;
        }
        Ok(())
    }

    /// Remove file if exists
    #[allow(dead_code)]
    pub fn remove_file(path: &str) -> Result<()> {
        if Path::new(path).exists() {
            fs::remove_file(path)?;
        }
        Ok(())
    }

    /// Copy file
    #[allow(dead_code)]
    pub fn copy_file(from: &str, to: &str) -> Result<()> {
        fs::copy(from, to)?;
        Ok(())
    }

    /// Read file line by line
    #[allow(dead_code)]
    pub fn read_lines<F>(path: &str, mut callback: F) -> Result<()>
    where
        F: FnMut(String) -> bool,
    {
        let file = File::open(path)?;
        let reader = io::BufReader::new(file);

        for line in reader.lines().map_while(std::result::Result::ok) {
            if !callback(line) {
                break;
            }
        }

        Ok(())
    }

    /// Get file modification time
    #[allow(dead_code)]
    pub fn modified_time(path: &str) -> Result<std::time::SystemTime> {
        fs::metadata(path)
            .and_then(|m| m.modified())
            .map_err(BondError::WriteConfigError)
    }

    /// Check if running as root
    #[allow(unreachable_code)]
    #[allow(dead_code)]
    pub fn is_root() -> bool {
        #[cfg(unix)]
        {
            return unsafe { libc::geteuid() == 0 };
        }
        // On non-Unix systems, assume not root
        #[cfg(not(unix))]
        false
    }

    /// Require root privileges
    #[allow(dead_code)]
    pub fn require_root() -> Result<()> {
        if !Self::is_root() {
            return Err(BondError::PermissionDenied);
        }
        Ok(())
    }
}

/// Safe path operations to prevent directory traversal
#[allow(dead_code)]
pub struct SafePath;

impl SafePath {
    /// Check if path is safe (no directory traversal)
    #[allow(dead_code)]
    pub fn is_safe(path: &str) -> bool {
        let path_obj = Path::new(path);

        // Check for .. components
        for component in path_obj.components() {
            if component == std::path::Component::ParentDir {
                return false;
            }
        }

        // Check for absolute paths with suspicious patterns
        let path_str = path.replace("\\", "/");
        if path_str.contains("../") || path_str.contains("..\\") {
            return false;
        }

        true
    }

    /// Join paths safely
    #[allow(dead_code)]
    pub fn join(base: &str, relative: &str) -> String {
        if Self::is_safe(relative) {
            Path::new(base).join(relative).to_string_lossy().to_string()
        } else {
            base.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_path() {
        assert!(SafePath::is_safe("ifcfg-eth0"));
        assert!(SafePath::is_safe("network-scripts/ifcfg-eth0"));
        assert!(!SafePath::is_safe("../etc/passwd"));
        assert!(!SafePath::is_safe("foo/../etc/passwd"));
    }

    #[test]
    fn test_safe_path_join() {
        let result = SafePath::join("/etc", "sysconfig/network-scripts");
        assert!(result.contains("network-scripts"));
    }
}
