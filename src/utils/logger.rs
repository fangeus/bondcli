use colored::Colorize;

pub fn info(msg: &str) {
    println!("{}", format!("[INFO] {}", msg).green());
}

pub fn warn(msg: &str) {
    println!("{}", format!("[WARNING] {}", msg).yellow());
}

pub fn error(msg: &str) {
    println!("{}", format!("[ERROR] {}", msg).red());
}

pub fn success(msg: &str) {
    println!("{}", format!("[SUCCESS] {}", msg).green());
}

#[allow(dead_code)]
pub fn dry_run(msg: &str) {
    println!("{}", format!("[DRY-RUN] {}", msg).cyan());
}

#[allow(dead_code)]
pub fn print_bond_config(config: &crate::bond::config::BondConfig) {
    println!("\n{}", "Bond Configuration:".bold());
    println!("  {}: {}", "Name".cyan(), config.name);
    println!(
        "  {}: {}. {}",
        "Mode".cyan(),
        config.mode.as_u8(),
        config.mode.name()
    );
    println!("  {}: {:?}", "Slaves".cyan(), config.slaves);
    println!("  {}: {}ms", "Miimon".cyan(), config.miimon);

    match &config.ip_config {
        crate::bond::config::IpConfig::Dhcp => {
            println!("  {}: DHCP", "IP Config".cyan());
        }
        crate::bond::config::IpConfig::Static {
            ip,
            netmask,
            gateway,
        } => {
            println!(
                "  {}: {}/{}",
                "IP Config".cyan(),
                ip,
                netmask_to_cidr(netmask)
            );
            if let Some(gw) = gateway {
                println!("  {}: {}", "Gateway".cyan(), gw);
            }
        }
    }

    if let Some(vlan_id) = config.vlan_id {
        println!("  {}: {}", "VLAN ID".cyan(), vlan_id);
    }

    if let Some(ref primary) = config.primary {
        println!("  {}: {}", "Primary".cyan(), primary);
    }
}

#[allow(dead_code)]
fn netmask_to_cidr(netmask: &std::net::Ipv4Addr) -> u8 {
    let bits = u32::from_be_bytes(netmask.octets());
    bits.count_ones() as u8
}

#[allow(dead_code)]
pub fn format_bond_summary(bond: &crate::bond::manager::BondSummary) -> String {
    let state_str = match bond.state {
        crate::bond::manager::BondState::Up => "UP".green(),
        crate::bond::manager::BondState::Down => "DOWN".red(),
        crate::bond::manager::BondState::Unknown => "UNKNOWN".yellow(),
    };

    format!(
        "{:<12} {:<15} {:<10} {:<10}",
        bond.name,
        format!("{}. {}", bond.mode.as_u8(), bond.mode.name()),
        bond.member_count,
        state_str
    )
}
