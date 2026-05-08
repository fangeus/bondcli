use bondcli::bond::config::{BondConfig, BondMode, IpConfig};

#[test]
fn test_bond_mode_from_u8_valid() {
    assert_eq!(BondMode::from_u8(0), Some(BondMode::BalanceRr));
    assert_eq!(BondMode::from_u8(1), Some(BondMode::ActiveBackup));
    assert_eq!(BondMode::from_u8(2), Some(BondMode::BalanceXor));
    assert_eq!(BondMode::from_u8(3), Some(BondMode::Broadcast));
    assert_eq!(BondMode::from_u8(4), Some(BondMode::Ieee8023ad));
    assert_eq!(BondMode::from_u8(5), Some(BondMode::BalanceTlb));
    assert_eq!(BondMode::from_u8(6), Some(BondMode::BalanceAlb));
}

#[test]
fn test_bond_mode_from_u8_invalid() {
    assert_eq!(BondMode::from_u8(7), None);
    assert_eq!(BondMode::from_u8(255), None);
    assert_eq!(BondMode::from_u8(10), None);
}

#[test]
fn test_bond_mode_as_u8() {
    assert_eq!(BondMode::BalanceRr.as_u8(), 0);
    assert_eq!(BondMode::ActiveBackup.as_u8(), 1);
    assert_eq!(BondMode::BalanceXor.as_u8(), 2);
    assert_eq!(BondMode::Broadcast.as_u8(), 3);
    assert_eq!(BondMode::Ieee8023ad.as_u8(), 4);
    assert_eq!(BondMode::BalanceTlb.as_u8(), 5);
    assert_eq!(BondMode::BalanceAlb.as_u8(), 6);
}

#[test]
fn test_bond_mode_require_lacp_warning() {
    assert!(!BondMode::BalanceRr.require_lacp_warning());
    assert!(!BondMode::ActiveBackup.require_lacp_warning());
    assert!(!BondMode::BalanceXor.require_lacp_warning());
    assert!(!BondMode::Broadcast.require_lacp_warning());
    assert!(BondMode::Ieee8023ad.require_lacp_warning()); // Only LACP
    assert!(!BondMode::BalanceTlb.require_lacp_warning());
    assert!(!BondMode::BalanceAlb.require_lacp_warning());
}

#[test]
fn test_ip_config_from_separate() {
    let ip = "192.168.1.100".parse().unwrap();
    let netmask = "255.255.255.0".parse().unwrap();
    let gateway = "192.168.1.1".parse().ok();

    let config = IpConfig::from_separate(ip, netmask, gateway);

    match config {
        IpConfig::Static { ip: i, netmask: n, gateway: g } => {
            assert_eq!(i.to_string(), "192.168.1.100");
            assert_eq!(n.to_string(), "255.255.255.0");
            assert!(g.is_some());
            assert_eq!(g.unwrap().to_string(), "192.168.1.1");
        }
        _ => panic!("Expected Static config"),
    }
}

#[test]
fn test_ip_config_get_gateway() {
    let gateway = "192.168.1.1".parse().ok();
    let config = IpConfig::from_separate(
        "192.168.1.100".parse().unwrap(),
        "255.255.255.0".parse().unwrap(),
        gateway,
    );

    assert!(config.get_gateway().is_some());
    assert_eq!(config.get_gateway().unwrap().to_string(), "192.168.1.1");
}

#[test]
fn test_ip_config_get_gateway_none() {
    let config = IpConfig::from_separate(
        "192.168.1.100".parse().unwrap(),
        "255.255.255.0".parse().unwrap(),
        None,
    );

    assert!(config.get_gateway().is_none());
}

#[test]
fn test_bond_config_default_mode() {
    let mode = BondMode::default();
    assert_eq!(mode, BondMode::ActiveBackup);
}

#[test]
fn test_bond_config_default_ip() {
    let ip_config = IpConfig::default();
    assert!(matches!(ip_config, IpConfig::Dhcp));
}

#[test]
fn test_bond_config_struct() {
    let config = BondConfig {
        name: "bond0".to_string(),
        mode: BondMode::ActiveBackup,
        slaves: vec!["eth0".to_string(), "eth1".to_string()],
        ip_config: IpConfig::Dhcp,
        vlan_id: Some(100),
        miimon: 100,
        primary: Some("eth0".to_string()),
    };

    assert_eq!(config.name, "bond0");
    assert_eq!(config.slaves.len(), 2);
    assert_eq!(config.vlan_id, Some(100));
    assert_eq!(config.miimon, 100);
}

#[test]
fn test_bond_mode_clone() {
    let mode = BondMode::ActiveBackup;
    let cloned = mode;
    assert_eq!(mode, cloned);
}

#[test]
fn test_ip_config_clone() {
    let ip_config = IpConfig::from_cidr("10.0.0.1/24", None).unwrap();
    let cloned = ip_config.clone();
    assert_eq!(ip_config, cloned);
}
