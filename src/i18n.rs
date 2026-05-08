use std::env;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Language {
    Chinese,
    English,
}

impl Language {
    #[allow(dead_code)]
    pub fn current() -> Language {
        detect_language()
    }
}

pub fn detect_language() -> Language {
    let lang = env::var("LANG")
        .or_else(|_| env::var("LC_ALL"))
        .unwrap_or_default();
    if lang.contains("zh_CN") || lang.contains("zh") {
        Language::Chinese
    } else {
        Language::English
    }
}

pub struct I18n;

impl I18n {
    // Bond name prompt
    pub fn bond_name_prompt() -> &'static str {
        match detect_language() {
            Language::Chinese => "请输入 Bond 名称 [bond0]:",
            Language::English => "Enter bond name [bond0]:",
        }
    }

    // Bond name validation error
    pub fn bond_name_invalid() -> &'static str {
        match detect_language() {
            Language::Chinese => "无效的接口名称，只允许字母、数字和下划线",
            Language::English => {
                "Invalid interface name, only letters, numbers and underscore allowed"
            }
        }
    }

    // Bond mode prompt
    pub fn bond_mode_prompt() -> &'static str {
        match detect_language() {
            Language::Chinese => "请选择 Bond 模式 [1]:",
            Language::English => "Select bond mode [1]:",
        }
    }

    // Mode list header
    pub fn bond_mode_list_header() -> &'static str {
        match detect_language() {
            Language::Chinese => "可用模式:",
            Language::English => "Available modes:",
        }
    }

    // Slave selection prompt
    pub fn slave_selection_prompt(needed: usize) -> String {
        match detect_language() {
            Language::Chinese => format!("请选择 {} 张网卡 (输入序号，多选用空格分隔):", needed),
            Language::English => format!(
                "Select {} interface(s) (enter index, space-separated for multiple):",
                needed
            ),
        }
    }

    // No interfaces available
    pub fn no_interfaces_available() -> &'static str {
        match detect_language() {
            Language::Chinese => "没有可用的网卡",
            Language::English => "No interfaces available",
        }
    }

    // IP configuration prompt
    pub fn ip_config_prompt() -> &'static str {
        match detect_language() {
            Language::Chinese => "请选择 IP 配置方式:",
            Language::English => "Select IP configuration:",
        }
    }

    // IP option DHCP
    pub fn ip_option_dhcp() -> &'static str {
        match detect_language() {
            Language::Chinese => "1) DHCP",
            Language::English => "1) DHCP",
        }
    }

    // IP option CIDR
    pub fn ip_option_cidr() -> &'static str {
        match detect_language() {
            Language::Chinese => "2) 静态 IP (CIDR 格式)",
            Language::English => "2) Static IP (CIDR format)",
        }
    }

    // IP option Separate
    pub fn ip_option_separate() -> &'static str {
        match detect_language() {
            Language::Chinese => "3) 静态 IP (分离式)",
            Language::English => "3) Static IP (separate)",
        }
    }

    // CIDR input prompt
    pub fn cidr_input_prompt() -> &'static str {
        match detect_language() {
            Language::Chinese => "请输入 IP/CIDR (如 192.168.1.100/24):",
            Language::English => "Enter IP/CIDR (e.g., 192.168.1.100/24):",
        }
    }

    // Gateway prompt
    pub fn gateway_prompt() -> &'static str {
        match detect_language() {
            Language::Chinese => "请输入网关地址 (可选，直接回车跳过):",
            Language::English => "Enter gateway address (optional, press Enter to skip):",
        }
    }

    // VLAN prompt
    pub fn vlan_prompt() -> &'static str {
        match detect_language() {
            Language::Chinese => "是否配置 VLAN?",
            Language::English => "Configure VLAN?",
        }
    }

    // VLAN ID prompt
    pub fn vlan_id_prompt() -> &'static str {
        match detect_language() {
            Language::Chinese => "请输入 VLAN ID (1-4094):",
            Language::English => "Enter VLAN ID (1-4094):",
        }
    }

    // VLAN info
    pub fn vlan_info() -> &'static str {
        match detect_language() {
            Language::Chinese => "注意: VLAN ID 与 IP 地址无关，仅用于创建 VLAN 子接口",
            Language::English => {
                "Note: VLAN ID is independent of IP address, used only for VLAN sub-interface"
            }
        }
    }

    // Miimon prompt
    pub fn miimon_prompt() -> &'static str {
        match detect_language() {
            Language::Chinese => "请输入 MII 监控间隔 (毫秒) [100]:",
            Language::English => "Enter MII monitoring interval (ms) [100]:",
        }
    }

    // Confirmation prompt
    pub fn confirmation_prompt() -> &'static str {
        match detect_language() {
            Language::Chinese => "确认创建? (y/N):",
            Language::English => "Confirm creation? (y/N):",
        }
    }

    // Summary header
    pub fn summary_header() -> &'static str {
        match detect_language() {
            Language::Chinese => "配置摘要:",
            Language::English => "Configuration Summary:",
        }
    }

    // User cancel message
    pub fn user_cancel() -> &'static str {
        match detect_language() {
            Language::Chinese => "[INFO] 用户取消操作",
            Language::English => "[INFO] Operation cancelled by user",
        }
    }

    // Warnings
    pub fn warn_single_slave() -> &'static str {
        match detect_language() {
            Language::Chinese => "单网卡 bond 存在风险，无法提供冗余或负载均衡能力",
            Language::English => "Single NIC bond has no redundancy or load balancing capability",
        }
    }

    pub fn warn_lacp_required() -> &'static str {
        match detect_language() {
            Language::Chinese => "802.3ad (LACP) 模式要求交换机已配置 LACP 端口聚合",
            Language::English => "802.3ad (LACP) mode requires LACP configured on the switch",
        }
    }

    pub fn warn_interface_has_ip() -> &'static str {
        match detect_language() {
            Language::Chinese => "该网卡已有 IP 配置，配置 bond 后将丢失原有 IP",
            Language::English => {
                "This interface has IP configuration, it will be lost after bonding"
            }
        }
    }

    #[allow(dead_code)]
    pub fn warn_primary_not_mode1() -> &'static str {
        match detect_language() {
            Language::Chinese => "--primary 选项仅在 mode 1 (ActiveBackup) 下有效",
            Language::English => "--primary option is only effective in mode 1 (ActiveBackup)",
        }
    }

    pub fn warn_networkmanager_disable() -> &'static str {
        match detect_language() {
            Language::Chinese => "工具将永久禁用 NetworkManager，如需恢复请执行: systemctl enable --now NetworkManager",
            Language::English => "This tool will permanently disable NetworkManager. To restore: systemctl enable --now NetworkManager",
        }
    }

    // Success messages
    pub fn success_bond_created() -> &'static str {
        match detect_language() {
            Language::Chinese => "[SUCCESS] Bond 创建成功",
            Language::English => "[SUCCESS] Bond created successfully",
        }
    }

    pub fn success_slave_added() -> &'static str {
        match detect_language() {
            Language::Chinese => "[SUCCESS] 成员添加成功",
            Language::English => "[SUCCESS] Slave added successfully",
        }
    }

    pub fn success_slave_removed() -> &'static str {
        match detect_language() {
            Language::Chinese => "[SUCCESS] 成员移除成功",
            Language::English => "[SUCCESS] Slave removed successfully",
        }
    }

    pub fn success_slave_replaced() -> &'static str {
        match detect_language() {
            Language::Chinese => "[SUCCESS] 成员替换成功",
            Language::English => "[SUCCESS] Slave replaced successfully",
        }
    }

    pub fn success_bond_deleted() -> &'static str {
        match detect_language() {
            Language::Chinese => "[SUCCESS] Bond 删除成功",
            Language::English => "[SUCCESS] Bond deleted successfully",
        }
    }

    // Network restart info
    pub fn network_restarting() -> &'static str {
        match detect_language() {
            Language::Chinese => "正在重启网络服务...",
            Language::English => "Restarting network service...",
        }
    }

    pub fn network_restart_success() -> &'static str {
        match detect_language() {
            Language::Chinese => "网络服务重启成功",
            Language::English => "Network service restarted successfully",
        }
    }

    #[allow(dead_code)]
    pub fn dry_run_prefix() -> &'static str {
        match detect_language() {
            Language::Chinese => "[DRY-RUN]",
            Language::English => "[DRY-RUN]",
        }
    }

    // List/Info formatting
    pub fn list_header_name() -> &'static str {
        match detect_language() {
            Language::Chinese => "名称",
            Language::English => "Name",
        }
    }

    pub fn list_header_mode() -> &'static str {
        match detect_language() {
            Language::Chinese => "模式",
            Language::English => "Mode",
        }
    }

    pub fn list_header_members() -> &'static str {
        match detect_language() {
            Language::Chinese => "成员数",
            Language::English => "Members",
        }
    }

    pub fn list_header_state() -> &'static str {
        match detect_language() {
            Language::Chinese => "状态",
            Language::English => "State",
        }
    }

    pub fn info_slaves() -> &'static str {
        match detect_language() {
            Language::Chinese => "成员:",
            Language::English => "Slaves:",
        }
    }

    pub fn info_primary() -> &'static str {
        match detect_language() {
            Language::Chinese => "主网卡:",
            Language::English => "Primary:",
        }
    }

    pub fn info_current_active() -> &'static str {
        match detect_language() {
            Language::Chinese => "当前活动:",
            Language::English => "Current Active:",
        }
    }

    pub fn info_active_suffix() -> &'static str {
        match detect_language() {
            Language::Chinese => "(活动)",
            Language::English => "(active)",
        }
    }
}
