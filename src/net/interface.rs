use std::fs;
use std::net::Ipv4Addr;
use std::path::Path;

use crate::error::{BondError, Result};

#[derive(Debug, Clone, PartialEq)]
pub struct NetworkInterface {
    pub name: String,
    pub mac: String,
    pub is_up: bool,
    pub ip: Option<Ipv4Addr>,
    pub is_bond_member: bool,
    pub master: Option<String>,
}

pub trait InterfaceDetector {
    fn get_available_interfaces(&self) -> Result<Vec<NetworkInterface>>;
    fn get_all_interfaces(&self) -> Result<Vec<NetworkInterface>>;
    #[allow(dead_code)]
    fn is_available(&self, name: &str) -> Result<bool>;
    #[allow(dead_code)]
    fn get_interface(&self, name: &str) -> Result<NetworkInterface>;
}

impl NetworkInterface {
    pub fn from_sysfs(name: &str) -> Result<Self> {
        let sysfs_path = format!("/sys/class/net/{}", name);

        // Check if interface exists
        if !Path::new(&sysfs_path).exists() {
            return Err(BondError::InterfaceNotFound(name.to_string()));
        }

        // Get MAC address
        let mac_path = format!("{}/address", sysfs_path);
        let mac = if Path::new(&mac_path).exists() {
            fs::read_to_string(&mac_path)
                .map(|m| m.trim().to_lowercase())
                .unwrap_or_else(|_| "00:00:00:00:00:00".to_string())
        } else {
            "00:00:00:00:00:00".to_string()
        };

        // Check if interface is up
        let operstate_path = format!("{}/operstate", sysfs_path);
        let is_up = if Path::new(&operstate_path).exists() {
            let state = fs::read_to_string(&operstate_path).unwrap_or_default();
            state.trim() == "up" || state.trim() == "unknown"
        } else {
            false
        };

        // Get IP address from /sys/class/net/{name}
        let ip = Self::get_ip_from_sysfs(name);

        // Check if it's a bond member
        let master_path = format!("{}/master", sysfs_path);
        let (is_bond_member, master) = if Path::new(&master_path).exists() {
            if let Ok(target) = fs::read_link(&master_path) {
                let master_name = target
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|s| s.to_string());
                (true, master_name)
            } else {
                (false, None)
            }
        } else {
            (false, None)
        };

        Ok(NetworkInterface {
            name: name.to_string(),
            mac,
            is_up,
            ip,
            is_bond_member,
            master,
        })
    }

    fn get_ip_from_sysfs(name: &str) -> Option<Ipv4Addr> {
        // Try to read IP from /sys/class/net/{name}/addr
        let addr_path = format!("/sys/class/net/{}/addr", name);
        if let Ok(addr) = fs::read_to_string(&addr_path) {
            let addr = addr.trim();
            if let Ok(ip) = addr.parse::<Ipv4Addr>() {
                return Some(ip);
            }
        }

        // Try alternative methods
        // Check ifcfg file
        let ifcfg_path = format!("/etc/sysconfig/network-scripts/ifcfg-{}", name);
        if let Ok(content) = fs::read_to_string(&ifcfg_path) {
            for line in content.lines() {
                if line.starts_with("IPADDR=") {
                    let ip_str = line.trim_start_matches("IPADDR=");
                    if let Ok(ip) = ip_str.parse::<Ipv4Addr>() {
                        return Some(ip);
                    }
                }
            }
        }

        None
    }

    pub fn is_available_for_bond(&self) -> bool {
        // Must be up and not already a member of another bond
        self.is_up && !self.is_bond_member
    }

    pub fn has_ip_config(&self) -> bool {
        self.ip.is_some()
    }
}

pub struct RealInterfaceDetector {
    sysfs_path: String,
}

impl RealInterfaceDetector {
    pub fn new() -> Self {
        RealInterfaceDetector {
            sysfs_path: "/sys/class/net".to_string(),
        }
    }

    fn list_interfaces(&self) -> Result<Vec<String>> {
        let mut interfaces = Vec::new();
        let path = Path::new(&self.sysfs_path);

        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                // Skip loopback and virtual interfaces
                if name != "lo" && !name.starts_with("docker") && !name.starts_with("veth") {
                    interfaces.push(name);
                }
            }
        }

        Ok(interfaces)
    }
}

impl Default for RealInterfaceDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl InterfaceDetector for RealInterfaceDetector {
    fn get_available_interfaces(&self) -> Result<Vec<NetworkInterface>> {
        let all = self.get_all_interfaces()?;
        Ok(all
            .into_iter()
            .filter(|i| i.is_available_for_bond())
            .collect())
    }

    fn get_all_interfaces(&self) -> Result<Vec<NetworkInterface>> {
        let names = self.list_interfaces()?;
        let mut interfaces = Vec::new();

        for name in names {
            if let Ok(iface) = NetworkInterface::from_sysfs(&name) {
                interfaces.push(iface);
            }
        }

        Ok(interfaces)
    }

    fn is_available(&self, name: &str) -> Result<bool> {
        let iface = NetworkInterface::from_sysfs(name)?;
        Ok(iface.is_available_for_bond())
    }

    fn get_interface(&self, name: &str) -> Result<NetworkInterface> {
        NetworkInterface::from_sysfs(name)
    }
}
