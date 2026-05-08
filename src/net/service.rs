use std::path::Path;
use std::process::Command;

use crate::error::{BondError, Result};

pub trait NetworkServiceManager {
    fn check_systemd(&self) -> Result<()>;
    fn disable_networkmanager(&self) -> Result<()>;
    fn restart_network(&self) -> Result<()>;
    #[allow(dead_code)]
    fn is_network_running(&self) -> Result<bool>;
}

pub struct RealNetworkServiceManager;

impl RealNetworkServiceManager {
    pub fn new() -> Self {
        RealNetworkServiceManager
    }

    fn run_systemctl(&self, args: &[&str]) -> Result<()> {
        let output = Command::new("systemctl")
            .args(args)
            .output()
            .map_err(|e| BondError::NetworkRestartError(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(BondError::NetworkRestartError(stderr.to_string()));
        }

        Ok(())
    }

    fn check_service_exists(&self, service: &str) -> bool {
        let output = Command::new("systemctl").args(["cat", service]).output();

        output.map(|o| o.status.success()).unwrap_or(false)
    }
}

impl Default for RealNetworkServiceManager {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkServiceManager for RealNetworkServiceManager {
    fn check_systemd(&self) -> Result<()> {
        // Check if systemctl exists
        if !Path::new("/usr/bin/systemctl").exists() {
            return Err(BondError::MissingSystemd);
        }
        Ok(())
    }

    fn disable_networkmanager(&self) -> Result<()> {
        // Stop NetworkManager
        self.run_systemctl(&["stop", "NetworkManager"])?;

        // Disable NetworkManager
        self.run_systemctl(&["disable", "NetworkManager"])?;

        crate::utils::logger::warn(crate::i18n::I18n::warn_networkmanager_disable());

        Ok(())
    }

    fn restart_network(&self) -> Result<()> {
        // Check if network.service exists
        if !self.check_service_exists("network.service") {
            return Err(BondError::NetworkRestartError(
                "network.service not found".to_string(),
            ));
        }

        crate::utils::logger::info(crate::i18n::I18n::network_restarting());

        self.run_systemctl(&["restart", "network"])?;

        crate::utils::logger::success(crate::i18n::I18n::network_restart_success());

        Ok(())
    }

    fn is_network_running(&self) -> Result<bool> {
        let output = Command::new("systemctl")
            .args(["is-active", "network"])
            .output()
            .map_err(|e| BondError::NetworkRestartError(e.to_string()))?;

        Ok(String::from_utf8_lossy(&output.stdout).trim() == "active")
    }
}
