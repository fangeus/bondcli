# bondcli

Linux Bonding Configuration Tool

A command-line tool written in Rust for configuring network interface bonding on RHEL-based Linux systems.

## Features

- Interactive and parameter modes
- Support for all 7 bonding modes (0-6)
- Dynamic member management (add/remove/replace slaves)
- VLAN configuration support
- CIDR and separate IP format support
- Automatic backup before configuration changes
- Automatic language detection (Chinese/English)
- Single static binary - no runtime dependencies

## Supported Operating Systems

- RHEL / CentOS 7, 8, 9
- Rocky Linux / AlmaLinux 8, 9

## Supported Architectures

- x86_64-unknown-linux-gnu
- aarch64-unknown-linux-gnu

## Installation

### Pre-built Binary (Recommended)

```bash
curl -L -o bondcli https://github.com/xxx/bondcli/releases/latest/download/bondcli-linux-x86_64
chmod +x bondcli
sudo mv bondcli /usr/local/bin/bondcli
```

Or use the Makefile:

```bash
sudo make install-prebuilt
```

### Build from Source

Requirements:
- Rust 1.70+ (install via rustup)
- Cargo

```bash
git clone https://github.com/xxx/bondcli.git
cd bondcli
make release
sudo make install
```

## Quick Start

### Interactive Mode

```bash
sudo bondcli create --interactive
```

### Parameter Mode

```bash
# Create a bond with DHCP
sudo bondcli create --name bond0 --slaves eth0 eth1 --mode 1 --dhcp

# Create a bond with static IP
sudo bondcli create --name bond0 --slaves eth0 eth1 --mode 1 --ip 192.168.1.100/24 --gateway 192.168.1.1

# Create a bond with VLAN
sudo bondcli create --name bond0 --slaves eth0 eth1 --mode 1 --dhcp --vlan-id 100
```

## Command Reference

### Create Bond

```bash
bondcli create [OPTIONS]

Options:
  -n, --name <NAME>        Bond interface name [default: bond0]
  -m, --mode <MODE>        Bond mode (0-6) [default: 1]
  -s, --slaves <SLIVES>    Slave interfaces (required)
  --dhcp                   Use DHCP for IP configuration
  --ip <IP/CIDR>           Static IP in CIDR format
  --gateway <IP>          Gateway IP address
  --vlan-id <ID>           VLAN ID (1-4094)
  --miimon <MS>            MII monitoring interval [default: 100]
  --primary <IFACE>       Primary slave (mode 1 only)
  -i, --interactive       Interactive mode
  --dry-run               Show what would be done
  --no-restart            Don't restart network service
```

### Add Slave

```bash
bondcli add <BOND> <SLAVE> [--dry-run] [--no-restart]
```

### Remove Slave

```bash
bondcli remove <BOND> <SLAVE> [--dry-run] [--no-restart]
```

### Replace Slave

```bash
bondcli replace <BOND> <OLD> <NEW> [--dry-run] [--no-restart]
```

### Delete Bond

```bash
bondcli delete <BOND> [--dry-run] [--no-restart]
```

### List Bonds

```bash
bondcli list
```

### Show Bond Info

```bash
bondcli info <BOND>
```

### Show Status

```bash
bondcli status
```

## Bond Modes

| Mode | Name | Description |
|------|------|-------------|
| 0 | Round-Robin | Sequential transmission on all slaves |
| 1 | Active-Backup | Only one slave active at a time |
| 2 | Balance-XOR | Transmit based on source/dest MAC XOR |
| 3 | Broadcast | Transmit on all slaves |
| 4 | 802.3ad (LACP) | IEEE 802.3ad Dynamic Link Aggregation |
| 5 | Balance-TLB | Transmit Load Balancing |
| 6 | Balance-ALB | Adaptive Load Balancing |

## Interactive Mode Flow

```
1. Enter bond name [bond0]
2. Select bond mode [1]
3. Select slave interfaces (at least 2)
4. Configure IP (DHCP / CIDR / Separate)
5. Configure VLAN (optional)
6. Set MII monitoring interval [100]
7. Confirm and create
```

## Common Scenarios

### Create Active-Backup Bond with DHCP

```bash
sudo bondcli create -n bond0 -s eth0 eth1 -m 1 --dhcp
```

### Create 802.3ad Bond with Static IP

```bash
sudo bondcli create -n bond0 -s eth0 eth1 -m 4 --ip 10.0.0.100/24 --gateway 10.0.0.1
```

### Add New Slave to Existing Bond

```bash
sudo bondcli add bond0 eth2
```

### Replace Failed Slave

```bash
sudo bondcli replace bond0 eth0 eth3
```

### Delete Bond

```bash
sudo bondcli delete bond0
```

## NetworkManager

**Warning**: This tool will permanently disable NetworkManager.

If you need to restore NetworkManager later:

```bash
sudo systemctl enable --now NetworkManager
```

## VLAN Configuration

VLAN ID and IP address are independent. The VLAN ID is used only to create a VLAN sub-interface.

Example: `--vlan-id 100` creates interface `bond0.100`

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Interface not found or already in use |
| 3 | Configuration file write error |
| 4 | Network service restart failed |
| 5 | Operation prohibited |
| 6 | User cancelled |

## Troubleshooting

### "Permission denied"

Run with sudo: bondcli requires root privileges.

### "Network service restart failed"

Check network service status:
```bash
systemctl status network
journalctl -u network -n 50
```

### "Interface not found"

Verify interface exists:
```bash
ip link show
ls /sys/class/net/
```

## Development

```bash
# Build
make build

# Run tests
make test

# Lint
make lint

# Build release
make release

# Clean
make clean
```

## License

MIT OR Apache-2.0
