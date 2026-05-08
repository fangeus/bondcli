use std::fmt;
use std::net::Ipv4Addr;

use clap::ValueEnum;

use crate::error::{BondError, Result};

#[derive(Debug, Clone, PartialEq)]
pub struct BondConfig {
    pub name: String,
    pub mode: BondMode,
    pub slaves: Vec<String>,
    pub ip_config: IpConfig,
    pub vlan_id: Option<u16>,
    pub miimon: u32,
    pub primary: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, ValueEnum)]
pub enum BondMode {
    BalanceRr = 0,
    ActiveBackup = 1,
    BalanceXor = 2,
    Broadcast = 3,
    Ieee8023ad = 4,
    BalanceTlb = 5,
    BalanceAlb = 6,
}

impl BondMode {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(BondMode::BalanceRr),
            1 => Some(BondMode::ActiveBackup),
            2 => Some(BondMode::BalanceXor),
            3 => Some(BondMode::Broadcast),
            4 => Some(BondMode::Ieee8023ad),
            5 => Some(BondMode::BalanceTlb),
            6 => Some(BondMode::BalanceAlb),
            _ => None,
        }
    }

    pub fn as_u8(&self) -> u8 {
        *self as u8
    }

    pub fn name(&self) -> &'static str {
        let lang = crate::i18n::detect_language();
        match (self, lang) {
            (BondMode::BalanceRr, crate::i18n::Language::Chinese) => "轮询 (Round-Robin)",
            (BondMode::ActiveBackup, crate::i18n::Language::Chinese) => "主备 (Active-Backup)",
            (BondMode::BalanceXor, crate::i18n::Language::Chinese) => "XOR 负载均衡",
            (BondMode::Broadcast, crate::i18n::Language::Chinese) => "广播",
            (BondMode::Ieee8023ad, crate::i18n::Language::Chinese) => "802.3ad (LACP)",
            (BondMode::BalanceTlb, crate::i18n::Language::Chinese) => "适配器传输负载均衡",
            (BondMode::BalanceAlb, crate::i18n::Language::Chinese) => "适配器负载均衡",
            (BondMode::BalanceRr, _) => "Round-Robin",
            (BondMode::ActiveBackup, _) => "Active-Backup",
            (BondMode::BalanceXor, _) => "Balance-XOR",
            (BondMode::Broadcast, _) => "Broadcast",
            (BondMode::Ieee8023ad, _) => "802.3ad (LACP)",
            (BondMode::BalanceTlb, _) => "Balance-TLB",
            (BondMode::BalanceAlb, _) => "Balance-ALB",
        }
    }

    pub fn description(&self) -> &'static str {
        let lang = crate::i18n::detect_language();
        match (self, lang) {
            (BondMode::BalanceRr, crate::i18n::Language::Chinese) => "轮询策略，顺序传输所有端口",
            (BondMode::ActiveBackup, crate::i18n::Language::Chinese) => {
                "主备策略，同一时间只有一个端口活动"
            }
            (BondMode::BalanceXor, crate::i18n::Language::Chinese) => "基于源/目的 MAC 地址异或",
            (BondMode::Broadcast, crate::i18n::Language::Chinese) => "广播所有端口",
            (BondMode::Ieee8023ad, crate::i18n::Language::Chinese) => {
                "IEEE 802.3ad 链路聚合 (需交换机配置 LACP)"
            }
            (BondMode::BalanceTlb, crate::i18n::Language::Chinese) => "传输负载均衡，基于当前负载",
            (BondMode::BalanceAlb, crate::i18n::Language::Chinese) => {
                "自适应负载均衡，包含 TLB + ARP 协商"
            }
            (BondMode::BalanceRr, _) => "Round-robin policy, transmits in sequential order",
            (BondMode::ActiveBackup, _) => "Active-backup policy, only one slave active at a time",
            (BondMode::BalanceXor, _) => "XOR policy, transmits based on source/dest MAC XOR",
            (BondMode::Broadcast, _) => "Broadcast policy, transmits on all slaves",
            (BondMode::Ieee8023ad, _) => {
                "IEEE 802.3ad Dynamic Link Aggregation (requires LACP on switch)"
            }
            (BondMode::BalanceTlb, _) => "Transmit Load Balancing, based on current load",
            (BondMode::BalanceAlb, _) => "Adaptive Load Balancing, TLB + ARP negotiation",
        }
    }

    pub fn require_lacp_warning(&self) -> bool {
        matches!(self, BondMode::Ieee8023ad)
    }

    #[allow(dead_code)]
    fn name_en(&self) -> &'static str {
        match self {
            BondMode::BalanceRr => "Round-Robin",
            BondMode::ActiveBackup => "Active-Backup",
            BondMode::BalanceXor => "Balance-XOR",
            BondMode::Broadcast => "Broadcast",
            BondMode::Ieee8023ad => "802.3ad (LACP)",
            BondMode::BalanceTlb => "Balance-TLB",
            BondMode::BalanceAlb => "Balance-ALB",
        }
    }

    #[allow(dead_code)]
    fn name_cn(&self) -> &'static str {
        match self {
            BondMode::BalanceRr => "轮询 (Round-Robin)",
            BondMode::ActiveBackup => "主备 (Active-Backup)",
            BondMode::BalanceXor => "XOR 负载均衡",
            BondMode::Broadcast => "广播",
            BondMode::Ieee8023ad => "802.3ad (LACP)",
            BondMode::BalanceTlb => "适配器传输负载均衡",
            BondMode::BalanceAlb => "适配器负载均衡",
        }
    }

    #[allow(dead_code)]
    fn description_en(&self) -> &'static str {
        match self {
            BondMode::BalanceRr => "Round-robin policy, transmits in sequential order",
            BondMode::ActiveBackup => "Active-backup policy, only one slave active at a time",
            BondMode::BalanceXor => "XOR policy, transmits based on source/dest MAC XOR",
            BondMode::Broadcast => "Broadcast policy, transmits on all slaves",
            BondMode::Ieee8023ad => {
                "IEEE 802.3ad Dynamic Link Aggregation (requires LACP on switch)"
            }
            BondMode::BalanceTlb => "Transmit Load Balancing, based on current load",
            BondMode::BalanceAlb => "Adaptive Load Balancing, TLB + ARP negotiation",
        }
    }

    #[allow(dead_code)]
    fn description_cn(&self) -> &'static str {
        match self {
            BondMode::BalanceRr => "轮询策略，顺序传输所有端口",
            BondMode::ActiveBackup => "主备策略，同一时间只有一个端口活动",
            BondMode::BalanceXor => "基于源/目的 MAC 地址异或",
            BondMode::Broadcast => "广播所有端口",
            BondMode::Ieee8023ad => "IEEE 802.3ad 链路聚合 (需交换机配置 LACP)",
            BondMode::BalanceTlb => "传输负载均衡，基于当前负载",
            BondMode::BalanceAlb => "自适应负载均衡，包含 TLB + ARP 协商",
        }
    }
}

impl fmt::Display for BondMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum IpConfig {
    Dhcp,
    Static {
        ip: Ipv4Addr,
        netmask: Ipv4Addr,
        gateway: Option<Ipv4Addr>,
    },
}

impl IpConfig {
    pub fn from_cidr(cidr: &str, gateway: Option<Ipv4Addr>) -> Result<Self> {
        let parts: Vec<&str> = cidr.split('/').collect();
        if parts.len() != 2 {
            return Err(BondError::InvalidCidrFormat(cidr.to_string()));
        }

        let ip: Ipv4Addr = parts[0]
            .parse()
            .map_err(|_| BondError::InvalidCidrFormat(cidr.to_string()))?;

        let prefix_len: u8 = parts[1]
            .parse()
            .map_err(|_| BondError::InvalidCidrFormat(cidr.to_string()))?;

        if prefix_len > 32 {
            return Err(BondError::InvalidCidrFormat(cidr.to_string()));
        }

        let netmask = Ipv4Addr::from(u32::MAX << (32 - prefix_len));

        Ok(IpConfig::Static {
            ip,
            netmask,
            gateway,
        })
    }

    #[allow(dead_code)]
    pub fn from_separate(ip: Ipv4Addr, netmask: Ipv4Addr, gateway: Option<Ipv4Addr>) -> Self {
        IpConfig::Static {
            ip,
            netmask,
            gateway,
        }
    }

    #[allow(dead_code)]
    pub fn is_dhcp(&self) -> bool {
        matches!(self, IpConfig::Dhcp)
    }

    #[allow(dead_code)]
    pub fn get_ip(&self) -> Option<&Ipv4Addr> {
        match self {
            IpConfig::Dhcp => None,
            IpConfig::Static { ip, .. } => Some(ip),
        }
    }

    #[allow(dead_code)]
    pub fn get_gateway(&self) -> Option<&Ipv4Addr> {
        match self {
            IpConfig::Dhcp => None,
            IpConfig::Static { gateway, .. } => gateway.as_ref(),
        }
    }
}

#[allow(clippy::derivable_impls)]
impl Default for BondMode {
    fn default() -> Self {
        BondMode::ActiveBackup
    }
}

#[allow(clippy::derivable_impls)]
impl Default for IpConfig {
    fn default() -> Self {
        IpConfig::Dhcp
    }
}


