use anyhow::Result;
use clap::Parser;

mod bond;
mod cli;
mod error;
mod i18n;
mod interactive;
mod net;
mod utils;

use bond::config::{BondConfig, BondMode, IpConfig};
use bond::manager::{BondManager, RealBondManager};
use cli::Cli;
use error::BondError;
use interactive::InteractiveState;
use utils::logger;

fn main() -> std::process::ExitCode {
    let cli = Cli::parse();

    if let Err(e) = run(cli) {
        match e.downcast_ref::<BondError>() {
            Some(bond_err) => {
                let exit_code = match bond_err {
                    BondError::InterfaceNotFound(_) => 2,
                    BondError::InterfaceAlreadyInUse(_, _) => 2,
                    BondError::InterfaceNotUp(_) => 2,
                    BondError::InsufficientInterfaces { .. } => 2,
                    BondError::BondNotFound(_) => 2,
                    BondError::WriteConfigError(_) => 3,
                    BondError::NetworkRestartError(_) => 4,
                    BondError::CannotDeleteLastSlave(_) => 5,
                    BondError::UserCancel => 6,
                    _ => 1,
                };
                logger::error(&bond_err.to_string());
                std::process::exit(exit_code);
            }
            None => {
                logger::error(&e.to_string());
                std::process::exit(1);
            }
        }
    }

    std::process::exit(0)
}

fn run(cli: Cli) -> Result<()> {
    match cli.command {
        cli::Command::Create(create) => {
            let manager = RealBondManager::new();

            if create.interactive {
                let mut state = InteractiveState::new();
                interactive::run_interactive_create(&mut state, &manager)?;
            } else {
                let config = build_config_from_args(&create)?;
                manager.create(&config, create.dry_run, create.no_restart)?;
            }
        }
        cli::Command::Add(add) => {
            let manager = RealBondManager::new();
            manager.add_slave(&add.bond, &add.slave, add.dry_run, add.no_restart)?;
        }
        cli::Command::Remove(remove) => {
            let manager = RealBondManager::new();
            manager.delete_slave(
                &remove.bond,
                &remove.slave,
                remove.dry_run,
                remove.no_restart,
            )?;
        }
        cli::Command::Replace(replace) => {
            let manager = RealBondManager::new();
            manager.replace_slave(
                &replace.bond,
                &replace.old,
                &replace.new,
                replace.dry_run,
                replace.no_restart,
            )?;
        }
        cli::Command::Delete(delete) => {
            let manager = RealBondManager::new();
            manager.delete_bond(&delete.bond, delete.dry_run, delete.no_restart)?;
        }
        cli::Command::List(_) => {
            let manager = RealBondManager::new();
            let bonds = manager.list_bonds()?;
            interactive::print_bond_list(&bonds);
        }
        cli::Command::Info(info) => {
            let manager = RealBondManager::new();
            let bond_info = manager.get_bond_info(&info.bond)?;
            interactive::print_bond_info(&bond_info);
        }
        cli::Command::Status(_) => {
            let manager = RealBondManager::new();
            let bonds = manager.list_bonds()?;
            interactive::print_bond_list(&bonds);
        }
    }

    Ok(())
}

fn build_config_from_args(create: &cli::Create) -> Result<BondConfig, BondError> {
    let name = create.name.clone().unwrap_or_else(|| "bond0".to_string());
    let mode = create.mode.unwrap_or(BondMode::ActiveBackup);
    let slaves = create.slaves.clone();
    let miimon = create.miimon.unwrap_or(100);
    let primary = create.primary.clone();
    let vlan_id = create.vlan_id;

    let ip_config = if create.dhcp {
        IpConfig::Dhcp
    } else if let Some(ref cidr) = create.ip {
        IpConfig::from_cidr(cidr, create.gateway)?
    } else {
        IpConfig::Dhcp
    };

    Ok(BondConfig {
        name,
        mode,
        slaves,
        ip_config,
        vlan_id,
        miimon,
        primary,
    })
}
