use std::fs;
use std::path::Path;

use chrono::NaiveDateTime;
use walkdir::WalkDir;

use crate::error::{BondError, Result};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct BackupInfo {
    pub interface: String,
    pub original_path: String,
    pub backup_path: String,
    pub timestamp: NaiveDateTime,
}

impl BackupInfo {
    #[allow(dead_code)]
    pub fn generate_backup_path(interface: &str) -> String {
        let timestamp = chrono::Local::now().format("%Y%m%d%H%M%S");
        format!("ifcfg-{}.bak.{}", interface, timestamp)
    }

    #[allow(dead_code)]
    pub fn parse_backup_path(path: &str) -> Option<Self> {
        // Pattern: ifcfg-{interface}.bak.{YYYYMMDDHHMMSS}
        let path_obj = Path::new(path);
        let filename = path_obj.file_name()?.to_str()?;

        if !filename.starts_with("ifcfg-") || !filename.contains(".bak.") {
            return None;
        }

        let rest = filename.strip_prefix("ifcfg-")?;
        let parts: Vec<&str> = rest.split(".bak.").collect();

        if parts.len() != 2 {
            return None;
        }

        let interface = parts[0].to_string();
        let timestamp_str = parts[1];

        // Parse timestamp
        let timestamp = NaiveDateTime::parse_from_str(timestamp_str, "%Y%m%d%H%M%S").ok()?;

        Some(BackupInfo {
            interface: interface.clone(),
            original_path: format!("/etc/sysconfig/network-scripts/ifcfg-{}", interface),
            backup_path: path.to_string(),
            timestamp,
        })
    }
}

#[allow(dead_code)]
pub trait BackupManager {
    fn backup(&self, interface: &str) -> Result<BackupInfo>;
    fn has_backup(&self, interface: &str) -> Result<bool>;
    fn list_backups(&self) -> Result<Vec<BackupInfo>>;
}

#[allow(dead_code)]
pub struct RealBackupManager {
    backup_dir: String,
    network_scripts_dir: String,
}

impl RealBackupManager {
    #[allow(dead_code)]
    pub fn new() -> Self {
        RealBackupManager {
            backup_dir: "/var/tmp/bondcli_backups".to_string(),
            network_scripts_dir: "/etc/sysconfig/network-scripts".to_string(),
        }
    }

    #[allow(dead_code)]
    fn ensure_backup_dir(&self) -> Result<()> {
        let path = Path::new(&self.backup_dir);
        if !path.exists() {
            fs::create_dir_all(path)?;
        }
        Ok(())
    }
}

impl Default for RealBackupManager {
    fn default() -> Self {
        Self::new()
    }
}

impl BackupManager for RealBackupManager {
    fn backup(&self, interface: &str) -> Result<BackupInfo> {
        let ifcfg_path = Path::new(&self.network_scripts_dir).join(format!("ifcfg-{}", interface));

        if !ifcfg_path.exists() {
            return Err(BondError::InterfaceNotFound(interface.to_string()));
        }

        self.ensure_backup_dir()?;

        let backup_filename = BackupInfo::generate_backup_path(interface);
        let backup_path = Path::new(&self.backup_dir).join(&backup_filename);

        fs::copy(&ifcfg_path, &backup_path)?;

        let timestamp = chrono::Local::now().naive_local();

        Ok(BackupInfo {
            interface: interface.to_string(),
            original_path: ifcfg_path.to_string_lossy().to_string(),
            backup_path: backup_path.to_string_lossy().to_string(),
            timestamp,
        })
    }

    fn has_backup(&self, interface: &str) -> Result<bool> {
        let _pattern = format!("ifcfg-{}.bak.*", interface);

        for entry in WalkDir::new(&self.backup_dir)
            .max_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with(&format!("ifcfg-{}.bak.", interface)) {
                        return Ok(true);
                    }
                }
            }
        }

        Ok(false)
    }

    fn list_backups(&self) -> Result<Vec<BackupInfo>> {
        let mut backups = Vec::new();

        for entry in WalkDir::new(&self.backup_dir)
            .max_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() {
                if let Some(backup_info) = BackupInfo::parse_backup_path(&path.to_string_lossy()) {
                    backups.push(backup_info);
                }
            }
        }

        // Sort by timestamp descending (newest first)
        backups.sort_by_key(|b| std::cmp::Reverse(b.timestamp));

        Ok(backups)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backup_path_generation() {
        let path = BackupInfo::generate_backup_path("eth0");
        assert!(path.starts_with("ifcfg-eth0.bak."));
        assert_eq!(path.len(), "ifcfg-eth0.bak.YYYYMMDDHHMMSS".len());
    }

    #[test]
    fn test_backup_path_parsing() {
        let path = "/var/tmp/bondcli_backups/ifcfg-eth0.bak.20260127143022";
        let info = BackupInfo::parse_backup_path(path).unwrap();
        assert_eq!(info.interface, "eth0");
    }

    #[test]
    fn test_invalid_backup_path() {
        let path = "/var/tmp/bondcli_backups/ifcfg-eth0";
        assert!(BackupInfo::parse_backup_path(path).is_none());
    }
}
