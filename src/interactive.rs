use std::io::{self, BufRead, Write};
use std::net::Ipv4Addr;

use crate::bond::config::{BondConfig, BondMode, IpConfig};
use crate::bond::manager::{BondInfo, BondManager, BondState, BondSummary, LinkState};
use crate::error::BondError;
use crate::i18n::I18n;
use crate::net::interface::{InterfaceDetector, NetworkInterface, RealInterfaceDetector};
use crate::utils::logger;

#[allow(dead_code)]
pub enum InteractiveStep {
    BondName,
    BondMode,
    SelectSlaves {
        selected: Vec<String>,
        needed: usize,
    },
    IpConfig,
    VlanConfig,
    MiimonConfig,
    Confirm,
}

pub struct InteractiveState {
    pub bond_name: String,
    pub bond_mode: BondMode,
    pub slaves: Vec<String>,
    pub ip_config: Option<IpConfig>,
    pub vlan_id: Option<u16>,
    pub miimon: u32,
    pub step: InteractiveStep,
}

impl InteractiveState {
    pub fn new() -> Self {
        InteractiveState {
            bond_name: "bond0".to_string(),
            bond_mode: BondMode::ActiveBackup,
            slaves: Vec::new(),
            ip_config: None,
            vlan_id: None,
            miimon: 100,
            step: InteractiveStep::BondName,
        }
    }
}

impl Default for InteractiveState {
    fn default() -> Self {
        Self::new()
    }
}

pub fn run_interactive_create<M: BondManager>(
    state: &mut InteractiveState,
    manager: &M,
) -> Result<(), BondError> {
    let interface_detector = RealInterfaceDetector::new();

    loop {
        match state.step {
            InteractiveStep::BondName => {
                let name = prompt_bond_name()?;
                state.bond_name = name;
                state.step = InteractiveStep::BondMode;
            }
            InteractiveStep::BondMode => {
                let mode = prompt_bond_mode(state.bond_mode)?;
                state.bond_mode = mode;
                let needed = if state.slaves.is_empty() { 2 } else { 0 };
                state.step = InteractiveStep::SelectSlaves {
                    selected: Vec::new(),
                    needed,
                };
            }
            InteractiveStep::SelectSlaves { needed, .. } => {
                if needed > 0 {
                    let available = interface_detector.get_available_interfaces()?;
                    let new_selected = prompt_select_slaves(&available, needed)?;
                    state.slaves.extend(new_selected);
                }

                // Check for single slave warning
                if state.slaves.len() == 1 {
                    logger::warn(I18n::warn_single_slave());
                }

                state.step = InteractiveStep::IpConfig;
            }
            InteractiveStep::IpConfig => {
                let ip_config = prompt_ip_config()?;
                state.ip_config = Some(ip_config);
                state.step = InteractiveStep::VlanConfig;
            }
            InteractiveStep::VlanConfig => {
                let vlan_id = prompt_vlan_config()?;
                state.vlan_id = vlan_id;
                state.step = InteractiveStep::MiimonConfig;
            }
            InteractiveStep::MiimonConfig => {
                let miimon = prompt_miimon(state.miimon)?;
                state.miimon = miimon;
                state.step = InteractiveStep::Confirm;
            }
            InteractiveStep::Confirm => {
                if !prompt_confirmation(state)? {
                    println!("{}", I18n::user_cancel());
                    return Err(BondError::UserCancel);
                }

                let config = BondConfig {
                    name: state.bond_name.clone(),
                    mode: state.bond_mode,
                    slaves: state.slaves.clone(),
                    ip_config: state.ip_config.clone().unwrap_or(IpConfig::Dhcp),
                    vlan_id: state.vlan_id,
                    miimon: state.miimon,
                    primary: None,
                };

                manager.create(&config, false, false)?;
                return Ok(());
            }
        }
    }
}

fn prompt_bond_name() -> Result<String, BondError> {
    print!("{} ", I18n::bond_name_prompt());
    io::stdout().flush().unwrap();

    let stdin = io::stdin();
    let mut line = String::new();
    stdin.lock().read_line(&mut line).unwrap();

    let name = line.trim();
    if name.is_empty() {
        return Ok("bond0".to_string());
    }

    if is_valid_interface_name(name) {
        Ok(name.to_string())
    } else {
        println!("{}", I18n::bond_name_invalid());
        prompt_bond_name()
    }
}

fn is_valid_interface_name(name: &str) -> bool {
    if name.is_empty() || name.len() > 15 {
        return false;
    }
    name.chars().all(|c| c.is_alphanumeric() || c == '_')
}

fn prompt_bond_mode(default: BondMode) -> Result<BondMode, BondError> {
    println!("{}", I18n::bond_mode_list_header());
    for i in 0u8..=6u8 {
        if let Some(mode) = BondMode::from_u8(i) {
            let marker = if mode == default { "*" } else { " " };
            println!("{} {}. {}", marker, i, mode.description());
        }
    }

    print!("{} ", I18n::bond_mode_prompt());
    io::stdout().flush().unwrap();

    let stdin = io::stdin();
    let mut line = String::new();
    stdin.lock().read_line(&mut line).unwrap();

    let input = line.trim();
    let mode_num: u8 = if input.is_empty() {
        default.as_u8()
    } else {
        input.parse().unwrap_or(default.as_u8())
    };

    match BondMode::from_u8(mode_num) {
        Some(mode) => {
            if mode.require_lacp_warning() {
                logger::warn(I18n::warn_lacp_required());
            }
            Ok(mode)
        }
        None => {
            logger::warn(&format!("Invalid mode {}, using default", mode_num));
            Ok(default)
        }
    }
}

fn prompt_select_slaves(
    available: &[NetworkInterface],
    needed: usize,
) -> Result<Vec<String>, BondError> {
    let up_interfaces: Vec<_> = available
        .iter()
        .filter(|iface| iface.is_up && iface.is_available_for_bond())
        .collect();

    if up_interfaces.is_empty() {
        println!("{}", I18n::no_interfaces_available());
        return Err(BondError::InsufficientInterfaces {
            required: needed,
            available: 0,
        });
    }

    println!("\n{}", I18n::bond_mode_list_header());
    for (i, iface) in up_interfaces.iter().enumerate() {
        let ip_str = iface
            .ip
            .map(|ip| ip.to_string())
            .unwrap_or_else(|| "-".to_string());
        println!(
            "  {}. {} (MAC: {}, IP: {})",
            i + 1,
            iface.name,
            iface.mac,
            ip_str
        );

        if iface.has_ip_config() {
            logger::warn(I18n::warn_interface_has_ip());
        }
    }

    print!("\n{} ", I18n::slave_selection_prompt(needed));
    io::stdout().flush().unwrap();

    let stdin = io::stdin();
    let mut line = String::new();
    stdin.lock().read_line(&mut line).unwrap();

    let selected: Vec<usize> = line
        .split_whitespace()
        .filter_map(|s| s.parse::<usize>().ok())
        .filter(|&i| i >= 1 && i <= up_interfaces.len())
        .map(|i| i - 1)
        .collect();

    if selected.len() < needed {
        println!(
            "Need at least {} interfaces, selected {}",
            needed,
            selected.len()
        );
        return prompt_select_slaves(available, needed);
    }

    let names: Vec<String> = selected
        .iter()
        .map(|&i| up_interfaces[i].name.clone())
        .collect();
    Ok(names)
}

fn prompt_ip_config() -> Result<IpConfig, BondError> {
    println!("\n{}", I18n::ip_config_prompt());
    println!("  {}", I18n::ip_option_dhcp());
    println!("  {}", I18n::ip_option_cidr());
    println!("  {}", I18n::ip_option_separate());

    print!("> ");
    io::stdout().flush().unwrap();

    let stdin = io::stdin();
    let mut line = String::new();
    stdin.lock().read_line(&mut line).unwrap();

    match line.trim() {
        "1" | "" => Ok(IpConfig::Dhcp),
        "2" => {
            print!("{} ", I18n::cidr_input_prompt());
            io::stdout().flush().unwrap();

            let mut cidr = String::new();
            stdin.lock().read_line(&mut cidr).unwrap();
            let cidr = cidr.trim();

            let gateway = prompt_gateway()?;

            IpConfig::from_cidr(cidr, gateway)
        }
        "3" => {
            print!("IP: ");
            io::stdout().flush().unwrap();
            let mut ip_str = String::new();
            stdin.lock().read_line(&mut ip_str).unwrap();
            let ip: Ipv4Addr = ip_str
                .trim()
                .parse()
                .map_err(|_| BondError::InvalidCidrFormat("Invalid IP address".to_string()))?;

            print!("Netmask: ");
            io::stdout().flush().unwrap();
            let mut nm_str = String::new();
            stdin.lock().read_line(&mut nm_str).unwrap();
            let netmask: Ipv4Addr = nm_str
                .trim()
                .parse()
                .map_err(|_| BondError::InvalidCidrFormat("Invalid netmask".to_string()))?;

            let gateway = prompt_gateway()?;

            Ok(IpConfig::Static {
                ip,
                netmask,
                gateway,
            })
        }
        _ => {
            println!("Invalid option, using DHCP");
            Ok(IpConfig::Dhcp)
        }
    }
}

fn prompt_gateway() -> Result<Option<Ipv4Addr>, BondError> {
    print!("{} ", I18n::gateway_prompt());
    io::stdout().flush().unwrap();

    let stdin = io::stdin();
    let mut line = String::new();
    stdin.lock().read_line(&mut line).unwrap();

    let input = line.trim();
    if input.is_empty() {
        return Ok(None);
    }

    match input.parse() {
        Ok(gw) => Ok(Some(gw)),
        Err(_) => Ok(None),
    }
}

fn prompt_vlan_config() -> Result<Option<u16>, BondError> {
    print!("{} (y/N): ", I18n::vlan_prompt());
    io::stdout().flush().unwrap();

    let stdin = io::stdin();
    let mut line = String::new();
    stdin.lock().read_line(&mut line).unwrap();

    if line.trim().to_lowercase() != "y" && line.trim().to_lowercase() != "yes" {
        return Ok(None);
    }

    println!("{}", I18n::vlan_info());

    print!("{} ", I18n::vlan_id_prompt());
    io::stdout().flush().unwrap();

    let mut vlan_str = String::new();
    stdin.lock().read_line(&mut vlan_str).unwrap();

    let vlan_id: u16 = vlan_str.trim().parse().unwrap_or(0);

    if !(1..=4094).contains(&vlan_id) {
        return Err(BondError::InvalidVlanId(vlan_id));
    }

    Ok(Some(vlan_id))
}

fn prompt_miimon(default: u32) -> Result<u32, BondError> {
    print!("{} ", I18n::miimon_prompt());
    io::stdout().flush().unwrap();

    let stdin = io::stdin();
    let mut line = String::new();
    stdin.lock().read_line(&mut line).unwrap();

    let input = line.trim();
    if input.is_empty() {
        return Ok(default);
    }

    match input.parse() {
        Ok(miimon) => Ok(miimon),
        Err(_) => Ok(default),
    }
}

fn prompt_confirmation(state: &InteractiveState) -> Result<bool, BondError> {
    println!("\n{}", I18n::summary_header());
    println!("  Name: {}", state.bond_name);
    println!(
        "  Mode: {}. {}",
        state.bond_mode.as_u8(),
        state.bond_mode.name()
    );
    println!("  Slaves: {}", state.slaves.join(", "));
    println!("  Miimon: {}ms", state.miimon);

    if let Some(ref ip_config) = state.ip_config {
        match ip_config {
            IpConfig::Dhcp => println!("  IP: DHCP"),
            IpConfig::Static {
                ip,
                netmask,
                gateway,
            } => {
                print!("  IP: {}/{}", ip, cidr_from_netmask(netmask));
                if let Some(gw) = gateway {
                    println!(", Gateway: {}", gw);
                } else {
                    println!();
                }
            }
        }
    }

    if let Some(vlan_id) = state.vlan_id {
        println!("  VLAN ID: {}", vlan_id);
    }

    print!("\n{} ", I18n::confirmation_prompt());
    io::stdout().flush().unwrap();

    let stdin = io::stdin();
    let mut line = String::new();
    stdin.lock().read_line(&mut line).unwrap();

    Ok(line.trim().to_lowercase() == "y" || line.trim().to_lowercase() == "yes")
}

fn cidr_from_netmask(netmask: &Ipv4Addr) -> u8 {
    let bits = u32::from_be_bytes(netmask.octets());
    bits.count_ones() as u8
}

pub fn print_bond_list(bonds: &[BondSummary]) {
    println!(
        "\n{:<12} {:<15} {:<10} {:<10}",
        I18n::list_header_name(),
        I18n::list_header_mode(),
        I18n::list_header_members(),
        I18n::list_header_state()
    );
    println!("{}", "-".repeat(50));

    if bonds.is_empty() {
        println!("No bonds found.");
        return;
    }

    for bond in bonds {
        let state_str = match bond.state {
            BondState::Up => "UP",
            BondState::Down => "DOWN",
            BondState::Unknown => "UNKNOWN",
        };
        println!(
            "{:<12} {:<15} {:<10} {:<10}",
            bond.name,
            format!("{}. {}", bond.mode.as_u8(), bond.mode.name()),
            bond.member_count,
            state_str
        );
    }
}

pub fn print_bond_info(bond_info: &BondInfo) {
    println!("\n=== {} ===", bond_info.name);
    println!(
        "Mode: {}. {}",
        bond_info.mode.as_u8(),
        bond_info.mode.name()
    );
    println!("Miimon: {}ms", bond_info.miimon);

    if let Some(primary) = &bond_info.primary {
        println!("{} {}", I18n::info_primary(), primary);
    }

    println!("\n{}", I18n::info_slaves());
    for slave in &bond_info.slaves {
        let active_str = if slave.is_active {
            format!(" {}", I18n::info_active_suffix())
        } else {
            String::new()
        };
        let state_str = match slave.state {
            LinkState::Up => "UP",
            LinkState::Down => "DOWN",
        };
        println!(
            "  - {} ({}: {}){}",
            slave.name, state_str, slave.link_ok, active_str
        );
    }

    if let Some(active) = &bond_info.current_active {
        println!("\n{} {}", I18n::info_current_active(), active);
    }
}
