use std::net::Ipv4Addr;

use crate::error::{BondError, Result};

#[allow(dead_code)]
pub struct InterfaceValidator;

impl InterfaceValidator {
    #[allow(dead_code)]
    pub fn validate_interface_name(name: &str) -> Result<()> {
        if name.is_empty() {
            return Err(BondError::InterfaceNotFound(name.to_string()));
        }

        if name.len() > 15 {
            return Err(BondError::ConflictingArguments(
                "Interface name too long (max 15 chars)".to_string(),
            ));
        }

        // Valid interface names: eth0, ens33, em1, etc.
        // Can contain letters, digits, and some special chars like -
        let valid = name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == ':');

        if !valid {
            return Err(BondError::ConflictingArguments(
                "Invalid interface name characters".to_string(),
            ));
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub fn validate_ip(ip: &str) -> Result<Ipv4Addr> {
        ip.parse::<Ipv4Addr>()
            .map_err(|_| BondError::InvalidCidrFormat(format!("Invalid IP address: {}", ip)))
    }

    #[allow(dead_code)]
    pub fn validate_netmask(netmask: &str) -> Result<Ipv4Addr> {
        let nm = Self::validate_ip(netmask)?;

        // Check if netmask is valid (contiguous bits)
        let nm_bits = u32::from_be_bytes(nm.octets());
        if nm_bits != 0 {
            let trailing = !nm_bits;
            if trailing != 0 && (trailing & (trailing + 1)) != 0 {
                return Err(BondError::InvalidCidrFormat(
                    "Invalid netmask (bits not contiguous)".to_string(),
                ));
            }
        }

        Ok(nm)
    }

    #[allow(dead_code)]
    pub fn validate_ip_with_netmask(ip: &Ipv4Addr, netmask: &Ipv4Addr) -> Result<()> {
        // Check if IP is in valid range
        let ip_bits = u32::from_be_bytes(ip.octets());
        let nm_bits = u32::from_be_bytes(netmask.octets());

        // Network address (IP & ~netmask) should not be zero
        let network = ip_bits & !nm_bits;
        if network == 0 {
            return Err(BondError::InvalidCidrFormat(
                "IP address is a network address".to_string(),
            ));
        }

        // Broadcast address (IP | netmask) should not be all ones
        let broadcast = ip_bits | nm_bits;
        if broadcast == u32::MAX {
            return Err(BondError::InvalidCidrFormat(
                "IP address is a broadcast address".to_string(),
            ));
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub fn validate_gateway(gateway: &Ipv4Addr, ip: &Ipv4Addr) -> Result<()> {
        // Basic validation: gateway should be in the same subnet
        let gw_bits = u32::from_be_bytes(gateway.octets());
        let ip_bits = u32::from_be_bytes(ip.octets());

        // Simple check: first octet should match for typical networks
        // More sophisticated validation would require netmask
        if (gw_bits >> 24) != (ip_bits >> 24) {
            crate::utils::logger::warn("Gateway appears to be in different subnet than IP address");
        }

        Ok(())
    }
}
