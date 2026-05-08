use crate::bond::config::{BondConfig, BondMode, IpConfig};
use crate::error::{BondError, Result};

#[allow(dead_code)]
pub struct BondValidator;

impl BondValidator {
    #[allow(dead_code)]
    pub fn validate_config(config: &BondConfig) -> Result<()> {
        Self::validate_name(&config.name)?;
        Self::validate_slaves(&config.slaves)?;
        Self::validate_vlan_id(config.vlan_id)?;
        Self::validate_primary(&config.primary, config.mode)?;
        Self::validate_ip_config(&config.ip_config)?;

        Ok(())
    }

    #[allow(dead_code)]
    pub fn validate_name(name: &str) -> Result<()> {
        if name.is_empty() {
            return Err(BondError::ConflictingArguments(
                "Bond name cannot be empty".to_string(),
            ));
        }

        if name.len() > 15 {
            return Err(BondError::ConflictingArguments(
                "Bond name too long (max 15 chars)".to_string(),
            ));
        }

        if !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(BondError::ConflictingArguments(
                "Bond name contains invalid characters".to_string(),
            ));
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub fn validate_slaves(slaves: &[String]) -> Result<()> {
        if slaves.is_empty() {
            return Err(BondError::ConflictingArguments(
                "At least one slave interface is required".to_string(),
            ));
        }

        if slaves.len() == 1 {
            crate::utils::logger::warn(crate::i18n::I18n::warn_single_slave());
        }

        let mut seen = std::collections::HashSet::new();
        for slave in slaves {
            if !seen.insert(slave) {
                return Err(BondError::ConflictingArguments(format!(
                    "Duplicate slave interface: {}",
                    slave
                )));
            }
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub fn validate_vlan_id(vlan_id: Option<u16>) -> Result<()> {
        if let Some(id) = vlan_id {
            if !(1..=4094).contains(&id) {
                return Err(BondError::InvalidVlanId(id));
            }
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub fn validate_primary(primary: &Option<String>, mode: BondMode) -> Result<()> {
        if primary.is_some() && mode != BondMode::ActiveBackup {
            crate::utils::logger::warn(crate::i18n::I18n::warn_primary_not_mode1());
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub fn validate_ip_config(ip_config: &IpConfig) -> Result<()> {
        match ip_config {
            IpConfig::Static { ip, netmask, .. } => {
                // Validate IP is not 0.0.0.0 or 255.255.255.255
                if ip.octets() == [0, 0, 0, 0] {
                    return Err(BondError::InvalidCidrFormat(
                        "Invalid IP address".to_string(),
                    ));
                }
                if ip.octets() == [255, 255, 255, 255] {
                    return Err(BondError::InvalidCidrFormat(
                        "Invalid IP address".to_string(),
                    ));
                }

                // Validate netmask is contiguous
                let nm = u32::from_be_bytes(netmask.octets());
                if nm != 0 && (nm & (nm - 1)) != 0 {
                    let trailing = !nm;
                    if (trailing & (trailing + 1)) != 0 {
                        return Err(BondError::InvalidCidrFormat(
                            "Invalid netmask (not contiguous)".to_string(),
                        ));
                    }
                }
            }
            IpConfig::Dhcp => {}
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub fn validate_cidr(cidr: &str) -> Result<(std::net::Ipv4Addr, std::net::Ipv4Addr)> {
        let parts: Vec<&str> = cidr.split('/').collect();
        if parts.len() != 2 {
            return Err(BondError::InvalidCidrFormat(cidr.to_string()));
        }

        let ip: std::net::Ipv4Addr = parts[0]
            .parse()
            .map_err(|_| BondError::InvalidCidrFormat(format!("Invalid IP: {}", parts[0])))?;

        let prefix_len: u8 = parts[1].parse().map_err(|_| {
            BondError::InvalidCidrFormat(format!("Invalid prefix length: {}", parts[1]))
        })?;

        if prefix_len > 32 {
            return Err(BondError::InvalidCidrFormat(format!(
                "Prefix length {} > 32",
                prefix_len
            )));
        }

        let netmask = std::net::Ipv4Addr::from(u32::MAX << (32 - prefix_len));

        Ok((ip, netmask))
    }
}
