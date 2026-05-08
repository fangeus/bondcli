use bondcli::bond::config::IpConfig;

#[test]
fn test_cidr_parsing_24() {
    let result = IpConfig::from_cidr("10.0.0.1/24", None);
    assert!(result.is_ok());

    let config = result.unwrap();
    match config {
        IpConfig::Static { ip, netmask, .. } => {
            assert_eq!(ip.to_string(), "10.0.0.1");
            assert_eq!(netmask.to_string(), "255.255.255.0");
        }
        _ => panic!("Expected Static config"),
    }
}

#[test]
fn test_cidr_parsing_16() {
    let result = IpConfig::from_cidr("192.168.1.100/16", None);
    assert!(result.is_ok());

    let config = result.unwrap();
    match config {
        IpConfig::Static { ip, netmask, .. } => {
            assert_eq!(ip.to_string(), "192.168.1.100");
            assert_eq!(netmask.to_string(), "255.255.0.0");
        }
        _ => panic!("Expected Static config"),
    }
}

#[test]
fn test_cidr_parsing_32() {
    let result = IpConfig::from_cidr("192.168.1.1/32", None);
    assert!(result.is_ok());

    let config = result.unwrap();
    match config {
        IpConfig::Static { ip, netmask, .. } => {
            assert_eq!(ip.to_string(), "192.168.1.1");
            assert_eq!(netmask.to_string(), "255.255.255.255");
        }
        _ => panic!("Expected Static config"),
    }
}

#[test]
fn test_cidr_parsing_8() {
    let result = IpConfig::from_cidr("10.0.0.1/8", None);
    assert!(result.is_ok());

    let config = result.unwrap();
    match config {
        IpConfig::Static { ip, netmask, .. } => {
            assert_eq!(ip.to_string(), "10.0.0.1");
            assert_eq!(netmask.to_string(), "255.0.0.0");
        }
        _ => panic!("Expected Static config"),
    }
}

#[test]
fn test_cidr_invalid_format_no_slash() {
    let result = IpConfig::from_cidr("192.168.1.100", None);
    assert!(result.is_err());
}

#[test]
fn test_cidr_invalid_format_bad_prefix() {
    let result = IpConfig::from_cidr("192.168.1.100/abc", None);
    assert!(result.is_err());
}

#[test]
fn test_cidr_invalid_format_prefix_too_large() {
    let result = IpConfig::from_cidr("192.168.1.100/33", None);
    assert!(result.is_err());
}

#[test]
fn test_cidr_invalid_format_bad_ip() {
    let result = IpConfig::from_cidr("999.999.999.999/24", None);
    assert!(result.is_err());
}

#[test]
fn test_cidr_with_gateway() {
    let gateway = "192.168.1.254".parse().ok();
    let result = IpConfig::from_cidr("192.168.1.100/24", gateway);
    assert!(result.is_ok());

    let config = result.unwrap();
    match config {
        IpConfig::Static { ip, netmask, gateway } => {
            assert_eq!(ip.to_string(), "192.168.1.100");
            assert_eq!(netmask.to_string(), "255.255.255.0");
            assert!(gateway.is_some());
            assert_eq!(gateway.unwrap().to_string(), "192.168.1.254");
        }
        _ => panic!("Expected Static config"),
    }
}

#[test]
fn test_ip_config_is_dhcp() {
    assert!(IpConfig::Dhcp.is_dhcp());
}

#[test]
fn test_ip_config_get_ip() {
    let static_config = IpConfig::from_cidr("10.0.0.1/24", None).unwrap();
    assert!(static_config.get_ip().is_some());
    assert_eq!(static_config.get_ip().unwrap().to_string(), "10.0.0.1");
}

#[test]
fn test_ip_config_get_ip_dhcp() {
    assert!(IpConfig::Dhcp.get_ip().is_none());
}
