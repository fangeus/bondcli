use bondcli::utils::backup::BackupInfo;

#[test]
fn test_backup_path_generation() {
    let path = BackupInfo::generate_backup_path("eth0");
    
    // Check format
    assert!(path.starts_with("ifcfg-eth0.bak."));
    assert_eq!(path.len(), "ifcfg-eth0.bak.YYYYMMDDHHMMSS".len());
    
    // Extract timestamp and verify it's parseable
    let timestamp_str = path.strip_prefix("ifcfg-eth0.bak.").unwrap();
    assert_eq!(timestamp_str.len(), 14); // YYYYMMDDHHMMSS
    
    // Should be all digits
    assert!(timestamp_str.chars().all(|c| c.is_ascii_digit()));
}

#[test]
fn test_backup_path_generation_different_interfaces() {
    let eth0_path = BackupInfo::generate_backup_path("eth0");
    let eth1_path = BackupInfo::generate_backup_path("eth1");
    
    assert!(eth0_path.starts_with("ifcfg-eth0.bak."));
    assert!(eth1_path.starts_with("ifcfg-eth1.bak."));
    assert_ne!(eth0_path, eth1_path);
}

#[test]
fn test_backup_path_parsing_valid() {
    let path = "/var/tmp/bondcli_backups/ifcfg-eth0.bak.20260127143022";
    let result = BackupInfo::parse_backup_path(path);
    
    assert!(result.is_some());
    
    let info = result.unwrap();
    assert_eq!(info.interface, "eth0");
    assert_eq!(info.original_path, "/etc/sysconfig/network-scripts/ifcfg-eth0");
    assert_eq!(info.backup_path, path);
}

#[test]
fn test_backup_path_parsing_without_directory() {
    let path = "ifcfg-eth1.bak.20260127143022";
    let result = BackupInfo::parse_backup_path(path);
    
    assert!(result.is_some());
    
    let info = result.unwrap();
    assert_eq!(info.interface, "eth1");
}

#[test]
fn test_backup_path_parsing_invalid_no_bak() {
    let path = "ifcfg-eth0";
    let result = BackupInfo::parse_backup_path(path);
    
    assert!(result.is_none());
}

#[test]
fn test_backup_path_parsing_invalid_wrong_format() {
    let path = "ifcfg-eth0.backup";
    let result = BackupInfo::parse_backup_path(path);
    
    assert!(result.is_none());
}

#[test]
fn test_backup_path_parsing_invalid_timestamp() {
    let path = "ifcfg-eth0.bak.20261327143022"; // Invalid month 13
    let result = BackupInfo::parse_backup_path(path);
    
    assert!(result.is_none());
}

#[test]
fn test_backup_path_parsing_interface_with_numbers() {
    let path = "ifcfg-ens33.bak.20260127143022";
    let result = BackupInfo::parse_backup_path(path);
    
    assert!(result.is_some());
    assert_eq!(result.unwrap().interface, "ens33");
}

#[test]
fn test_backup_path_parsing_interface_with_dash() {
    let path = "ifcfg-eth-test0.bak.20260127143022";
    let result = BackupInfo::parse_backup_path(path);
    
    assert!(result.is_some());
    assert_eq!(result.unwrap().interface, "eth-test0");
}
