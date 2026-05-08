use clap::{Parser, Subcommand};
use std::net::Ipv4Addr;

use crate::bond::config::BondMode;

#[derive(Parser, Debug)]
#[command(
    name = "bondcli",
    about = "Linux Bonding Configuration Tool",
    long_about = "A command-line tool for configuring network interface bonding on RHEL-based Linux systems."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    Create(Create),
    Add(Add),
    Remove(Remove),
    Replace(Replace),
    Delete(Delete),
    List(List),
    Info(Info),
    Status(Status),
}

#[derive(Parser, Debug)]
#[command(about = "Create a new bond interface")]
pub struct Create {
    /// Bond interface name
    #[arg(short, long, default_value = "bond0")]
    pub name: Option<String>,

    /// Bond mode (0-6)
    #[arg(short, long)]
    pub mode: Option<BondMode>,

    /// Slave interfaces (at least 2 required for most modes)
    #[arg(short, long, required = true, num_args = 1..)]
    pub slaves: Vec<String>,

    /// Use DHCP for IP configuration
    #[arg(long)]
    pub dhcp: bool,

    /// IP address in CIDR format (e.g., 192.168.1.100/24)
    #[arg(long)]
    pub ip: Option<String>,

    /// Gateway IP address (only for static IP mode)
    #[arg(long)]
    pub gateway: Option<Ipv4Addr>,

    /// VLAN ID (1-4094)
    #[arg(long)]
    pub vlan_id: Option<u16>,

    /// MII monitoring interval in milliseconds
    #[arg(long, default_value = "100")]
    pub miimon: Option<u32>,

    /// Primary slave interface (for mode 1)
    #[arg(long)]
    pub primary: Option<String>,

    /// Run in interactive mode
    #[arg(short, long)]
    pub interactive: bool,

    /// Dry run (show what would be done without making changes)
    #[arg(long)]
    pub dry_run: bool,

    /// Do not restart network service
    #[arg(long)]
    pub no_restart: bool,
}

#[derive(Parser, Debug)]
#[command(about = "Add a slave interface to an existing bond")]
pub struct Add {
    /// Bond interface name
    #[arg(required = true)]
    pub bond: String,

    /// Slave interface name
    #[arg(required = true)]
    pub slave: String,

    /// Dry run
    #[arg(long)]
    pub dry_run: bool,

    /// Do not restart network service
    #[arg(long)]
    pub no_restart: bool,
}

#[derive(Parser, Debug)]
#[command(about = "Remove a slave interface from a bond")]
pub struct Remove {
    /// Bond interface name
    #[arg(required = true)]
    pub bond: String,

    /// Slave interface name
    #[arg(required = true)]
    pub slave: String,

    /// Dry run
    #[arg(long)]
    pub dry_run: bool,

    /// Do not restart network service
    #[arg(long)]
    pub no_restart: bool,
}

#[derive(Parser, Debug)]
#[command(about = "Replace a slave interface in a bond")]
pub struct Replace {
    /// Bond interface name
    #[arg(required = true)]
    pub bond: String,

    /// Old slave interface name
    #[arg(required = true)]
    pub old: String,

    /// New slave interface name
    #[arg(required = true)]
    pub new: String,

    /// Dry run
    #[arg(long)]
    pub dry_run: bool,

    /// Do not restart network service
    #[arg(long)]
    pub no_restart: bool,
}

#[derive(Parser, Debug)]
#[command(about = "Delete a bond interface")]
pub struct Delete {
    /// Bond interface name
    #[arg(required = true)]
    pub bond: String,

    /// Dry run
    #[arg(long)]
    pub dry_run: bool,

    /// Do not restart network service
    #[arg(long)]
    pub no_restart: bool,
}

#[derive(Parser, Debug)]
#[command(about = "List all bond interfaces")]
pub struct List {}

#[derive(Parser, Debug)]
#[command(about = "Show detailed information about a bond")]
pub struct Info {
    /// Bond interface name
    #[arg(required = true)]
    pub bond: String,
}

#[derive(Parser, Debug)]
#[command(about = "Show status of all bonds")]
pub struct Status {}
