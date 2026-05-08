use std::fs;
use std::io::Write;
use std::path::Path;

use crate::bond::config::{BondConfig, BondMode, IpConfig};
use crate::error::{BondError, Result};
use crate::net::service::NetworkServiceManager;

pub trait BondManager {
    fn create(&self, config: &BondConfig, dry_run: bool, no_restart: bool) -> Result<()>;
    fn add_slave(&self, bond: &str, slave: &str, dry_run: bool, no_restart: bool) -> Result<()>;
    fn delete_slave(&self, bond: &str, slave: &str, dry_run: bool, no_restart: bool) -> Result<()>;
    fn replace_slave(
        &self,
        bond: &str,
        old: &str,
        new: &str,
        dry_run: bool,
        no_restart: bool,
    ) -> Result<()>;
    fn delete_bond(&self, bond: &str, dry_run: bool, no_restart: bool) -> Result<()>;
    fn get_bond_info(&self, bond: &str) -> Result<BondInfo>;
    fn list_bonds(&self) -> Result<Vec<BondSummary>>;
}

#[derive(Debug, Clone)]
pub struct BondInfo {
    pub name: String,
    pub mode: BondMode,
    pub miimon: u32,
    pub primary: Option<String>,
    pub slaves: Vec<SlaveInfo>,
    pub current_active: Option<String>,
    pub overall_state: BondState,
}

#[derive(Debug, Clone)]
pub struct SlaveInfo {
    pub name: String,
    pub state: LinkState,
    pub link_ok: bool,
    pub is_active: bool,
}

#[derive(Debug, Clone)]
pub struct BondSummary {
    pub name: String,
    pub member_count: usize,
    pub mode: BondMode,
    pub state: BondState,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum BondState {
    Up,
    Down,
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LinkState {
    Up,
    Down,
}

pub struct RealBondManager {
    ifcfg_path: String,
}

impl RealBondManager {
    pub fn new() -> Self {
        RealBondManager {
            ifcfg_path: "/etc/sysconfig/network-scripts".to_string(),
        }
    }

    fn generate_bond_config(&self, config: &BondConfig) -> String {
        let mut content = String::new();

        content.push_str(&format!("DEVICE={}\n", config.name));
        content.push_str(&format!("NAME={}\n", config.name));
        content.push_str("TYPE=Bond\n");
        content.push_str("BONDING_MASTER=yes\n");
        content.push_str("ONBOOT=yes\n");

        match &config.ip_config {
            IpConfig::Dhcp => {
                content.push_str("BOOTPROTO=dhcp\n");
            }
            IpConfig::Static {
                ip,
                netmask,
                gateway,
            } => {
                content.push_str("BOOTPROTO=none\n");
                content.push_str(&format!("IPADDR={}\n", ip));
                content.push_str(&format!("NETMASK={}\n", netmask));
                if let Some(gw) = gateway {
                    content.push_str(&format!("GATEWAY={}\n", gw));
                }
            }
        }

        let bonding_opts = if let Some(ref primary) = config.primary {
            if config.mode == BondMode::ActiveBackup {
                format!(
                    "mode={} miimon={} primary={}",
                    config.mode.as_u8(),
                    config.miimon,
                    primary
                )
            } else {
                format!("mode={} miimon={}", config.mode.as_u8(), config.miimon)
            }
        } else {
            format!("mode={} miimon={}", config.mode.as_u8(), config.miimon)
        };
        content.push_str(&format!("BONDING_OPTS=\"{}\"\n", bonding_opts));

        content
    }

    fn generate_slave_config(&self, slave: &str, master: &str) -> String {
        let mut content = String::new();
        content.push_str(&format!("DEVICE={}\n", slave));
        content.push_str("ONBOOT=yes\n");
        content.push_str(&format!("MASTER={}\n", master));
        content.push_str("SLAVE=yes\n");
        content
    }

    fn generate_vlan_config(&self, bond: &str, vlan_id: u16, ip_config: &IpConfig) -> String {
        let device_name = format!("{}.{}", bond, vlan_id);
        let mut content = String::new();

        content.push_str(&format!("DEVICE={}\n", device_name));
        content.push_str("VLAN=yes\n");
        content.push_str("ONBOOT=yes\n");

        match ip_config {
            IpConfig::Dhcp => {
                content.push_str("BOOTPROTO=dhcp\n");
            }
            IpConfig::Static {
                ip,
                netmask,
                gateway,
            } => {
                content.push_str("BOOTPROTO=none\n");
                content.push_str(&format!("IPADDR={}\n", ip));
                content.push_str(&format!("NETMASK={}\n", netmask));
                if let Some(gw) = gateway {
                    content.push_str(&format!("GATEWAY={}\n", gw));
                }
            }
        }

        content
    }

    fn write_config_file(&self, filename: &str, content: &str) -> Result<()> {
        let path = Path::new(&self.ifcfg_path);
        let target = path.join(filename);
        let temp = path.join(format!(".{}.tmp", filename));

        let mut file = fs::File::create(&temp)?;
        file.write_all(content.as_bytes())?;
        file.sync_all()?;

        fs::rename(&temp, &target)?;

        Ok(())
    }

    fn backup_interface(&self, iface: &str) -> Result<()> {
        let path = Path::new(&self.ifcfg_path).join(format!("ifcfg-{}", iface));
        if path.exists() {
            let timestamp = chrono::Local::now().format("%Y%m%d%H%M%S");
            let backup_name = format!("ifcfg-{}.bak.{}", iface, timestamp);
            let backup_path = Path::new(&self.ifcfg_path).join(&backup_name);
            fs::copy(&path, &backup_path)?;
        }
        Ok(())
    }

    fn disable_networkmanager(&self) -> Result<()> {
        let service = crate::net::service::RealNetworkServiceManager::new();
        service.disable_networkmanager()?;
        Ok(())
    }

    fn restart_network(&self) -> Result<()> {
        let service = crate::net::service::RealNetworkServiceManager::new();
        service.restart_network()?;
        Ok(())
    }
}

impl Default for RealBondManager {
    fn default() -> Self {
        Self::new()
    }
}

impl BondManager for RealBondManager {
    fn create(&self, config: &BondConfig, dry_run: bool, no_restart: bool) -> Result<()> {
        // Check systemd
        let service = crate::net::service::RealNetworkServiceManager::new();
        service.check_systemd()?;

        // Check if already exists
        let bond_path = Path::new(&self.ifcfg_path).join(format!("ifcfg-{}", config.name));
        if bond_path.exists() {
            return Err(BondError::ConflictingArguments(format!(
                "Bond {} already exists",
                config.name
            )));
        }

        // Validate slaves
        for slave in &config.slaves {
            let slave_path = Path::new(&self.ifcfg_path).join(format!("ifcfg-{}", slave));
            if !slave_path.exists() {
                return Err(BondError::InterfaceNotFound(slave.clone()));
            }
        }

        // Generate and write bond config
        let bond_config = self.generate_bond_config(config);
        if dry_run {
            println!("[DRY-RUN] Would create ifcfg-{}:", config.name);
            println!("{}", bond_config);
        } else {
            self.write_config_file(&format!("ifcfg-{}", config.name), &bond_config)?;

            // Backup and configure slaves
            for slave in &config.slaves {
                self.backup_interface(slave)?;
                let slave_config = self.generate_slave_config(slave, &config.name);
                self.write_config_file(&format!("ifcfg-{}", slave), &slave_config)?;
            }

            // Create VLAN config if needed
            if let Some(vlan_id) = config.vlan_id {
                let vlan_config =
                    self.generate_vlan_config(&config.name, vlan_id, &config.ip_config);
                self.write_config_file(
                    &format!("ifcfg-{}.{}", config.name, vlan_id),
                    &vlan_config,
                )?;
            }

            // Disable NetworkManager and restart network
            if !no_restart {
                self.disable_networkmanager()?;
                self.restart_network()?;
            }
        }

        crate::utils::logger::success(crate::i18n::I18n::success_bond_created());
        Ok(())
    }

    fn add_slave(&self, bond: &str, slave: &str, dry_run: bool, no_restart: bool) -> Result<()> {
        let service = crate::net::service::RealNetworkServiceManager::new();
        service.check_systemd()?;

        // Check slave exists
        let slave_path = Path::new(&self.ifcfg_path).join(format!("ifcfg-{}", slave));
        if !slave_path.exists() {
            return Err(BondError::InterfaceNotFound(slave.to_string()));
        }

        if dry_run {
            println!("[DRY-RUN] Would add {} to bond {}", slave, bond);
        } else {
            self.backup_interface(slave)?;
            let slave_config = self.generate_slave_config(slave, bond);
            self.write_config_file(&format!("ifcfg-{}", slave), &slave_config)?;

            if !no_restart {
                self.restart_network()?;
            }
        }

        crate::utils::logger::success(crate::i18n::I18n::success_slave_added());
        Ok(())
    }

    fn delete_slave(&self, bond: &str, slave: &str, dry_run: bool, no_restart: bool) -> Result<()> {
        let service = crate::net::service::RealNetworkServiceManager::new();
        service.check_systemd()?;

        // Get current slaves count
        let bond_info = self.get_bond_info(bond)?;
        if bond_info.slaves.len() <= 1 {
            return Err(BondError::CannotDeleteLastSlave(bond.to_string()));
        }

        if dry_run {
            println!("[DRY-RUN] Would remove {} from bond {}", slave, bond);
        } else {
            // Reset slave config
            let mut content = String::new();
            content.push_str(&format!("DEVICE={}\n", slave));
            content.push_str("ONBOOT=yes\n");
            content.push_str("BOOTPROTO=dhcp\n");
            self.write_config_file(&format!("ifcfg-{}", slave), &content)?;

            if !no_restart {
                self.restart_network()?;
            }
        }

        crate::utils::logger::success(crate::i18n::I18n::success_slave_removed());
        Ok(())
    }

    fn replace_slave(
        &self,
        bond: &str,
        old: &str,
        new: &str,
        dry_run: bool,
        no_restart: bool,
    ) -> Result<()> {
        let service = crate::net::service::RealNetworkServiceManager::new();
        service.check_systemd()?;

        // Check new slave exists
        let new_path = Path::new(&self.ifcfg_path).join(format!("ifcfg-{}", new));
        if !new_path.exists() {
            return Err(BondError::InterfaceNotFound(new.to_string()));
        }

        if dry_run {
            println!(
                "[DRY-RUN] Would replace {} with {} in bond {}",
                old, new, bond
            );
        } else {
            // Remove old slave
            self.backup_interface(old)?;
            let old_content = format!("DEVICE={}\nONBOOT=yes\nBOOTPROTO=dhcp\n", old);
            self.write_config_file(&format!("ifcfg-{}", old), &old_content)?;

            // Add new slave
            self.backup_interface(new)?;
            let new_config = self.generate_slave_config(new, bond);
            self.write_config_file(&format!("ifcfg-{}", new), &new_config)?;

            if !no_restart {
                self.restart_network()?;
            }
        }

        crate::utils::logger::success(crate::i18n::I18n::success_slave_replaced());
        Ok(())
    }

    fn delete_bond(&self, bond: &str, dry_run: bool, no_restart: bool) -> Result<()> {
        let service = crate::net::service::RealNetworkServiceManager::new();
        service.check_systemd()?;

        let bond_path = Path::new(&self.ifcfg_path).join(format!("ifcfg-{}", bond));
        if !bond_path.exists() {
            return Err(BondError::BondNotFound(bond.to_string()));
        }

        if dry_run {
            println!("[DRY-RUN] Would delete bond {}", bond);
        } else {
            // Get slaves and reset them
            let bond_info = self.get_bond_info(bond)?;
            for slave_info in &bond_info.slaves {
                self.backup_interface(&slave_info.name)?;
                let content = format!("DEVICE={}\nONBOOT=yes\nBOOTPROTO=dhcp\n", slave_info.name);
                self.write_config_file(&format!("ifcfg-{}", slave_info.name), &content)?;
            }

            // Delete bond config
            fs::remove_file(&bond_path)?;

            if !no_restart {
                self.restart_network()?;
            }
        }

        crate::utils::logger::success(crate::i18n::I18n::success_bond_deleted());
        Ok(())
    }

    fn get_bond_info(&self, bond: &str) -> Result<BondInfo> {
        let bond_path = Path::new(&self.ifcfg_path).join(format!("ifcfg-{}", bond));
        if !bond_path.exists() {
            return Err(BondError::BondNotFound(bond.to_string()));
        }

        let content = fs::read_to_string(&bond_path)?;

        // Parse mode from BONDING_OPTS
        let mut mode = BondMode::ActiveBackup;
        let mut miimon = 100u32;
        let mut primary = None;

        for line in content.lines() {
            if line.starts_with("BONDING_OPTS=") {
                let opts = line.trim_start_matches("BONDING_OPTS=").trim_matches('"');
                for opt in opts.split_whitespace() {
                    if opt.starts_with("mode=") {
                        if let Ok(m) = opt.trim_start_matches("mode=").parse::<u8>() {
                            mode = BondMode::from_u8(m).unwrap_or(BondMode::ActiveBackup);
                        }
                    } else if opt.starts_with("miimon=") {
                        if let Ok(m) = opt.trim_start_matches("miimon=").parse::<u32>() {
                            miimon = m;
                        }
                    } else if opt.starts_with("primary=") {
                        primary = Some(opt.trim_start_matches("primary=").to_string());
                    }
                }
            }
        }

        // Find slaves
        let mut slaves = Vec::new();
        let entries = fs::read_dir(&self.ifcfg_path)?;
        for entry in entries {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("ifcfg-") && name != format!("ifcfg-{}", bond) {
                let slave_name = name.trim_start_matches("ifcfg-");
                if let Ok(slave_content) = fs::read_to_string(entry.path()) {
                    if slave_content.contains(&format!("MASTER={}", bond)) {
                        let state = if slave_content.contains("SLAVE=yes") {
                            LinkState::Up
                        } else {
                            LinkState::Down
                        };
                        let link_ok = true;
                        let is_active = slave_content.contains("PRIMARY=yes");
                        slaves.push(SlaveInfo {
                            name: slave_name.to_string(),
                            state,
                            link_ok,
                            is_active,
                        });
                    }
                }
            }
        }

        let current_active = slaves.iter().find(|s| s.is_active).map(|s| s.name.clone());
        let overall_state = if slaves.iter().any(|s| s.state == LinkState::Up) {
            BondState::Up
        } else {
            BondState::Down
        };

        Ok(BondInfo {
            name: bond.to_string(),
            mode,
            miimon,
            primary,
            slaves,
            current_active,
            overall_state,
        })
    }

    fn list_bonds(&self) -> Result<Vec<BondSummary>> {
        let mut bonds = Vec::new();
        let entries = fs::read_dir(&self.ifcfg_path)?;

        for entry in entries {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();

            if name.starts_with("ifcfg-bond") && !name.contains('.') {
                let bond_name = name.trim_start_matches("ifcfg-");
                if let Ok(info) = self.get_bond_info(bond_name) {
                    bonds.push(BondSummary {
                        name: bond_name.to_string(),
                        member_count: info.slaves.len(),
                        mode: info.mode,
                        state: info.overall_state,
                    });
                }
            }
        }

        Ok(bonds)
    }
}
