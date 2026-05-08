use thiserror::Error;

#[derive(Error, Debug)]
pub enum BondError {
    #[error("网卡 {0} 不存在")]
    InterfaceNotFound(String),

    #[error("网卡 {0} 已被 bond {1} 使用")]
    #[allow(dead_code)]
    InterfaceAlreadyInUse(String, String),

    #[error("网卡 {0} 状态为 DOWN，不可用于 bonding")]
    #[allow(dead_code)]
    InterfaceNotUp(String),

    #[error("无可用的网卡，需要 {required} 张，但只有 {available} 张可用")]
    InsufficientInterfaces { required: usize, available: usize },

    #[error("无法删除 bond {0} 的最后一个成员")]
    CannotDeleteLastSlave(String),

    #[error("bond {0} 不存在")]
    BondNotFound(String),

    #[error("写入配置文件失败: {0}")]
    WriteConfigError(#[from] std::io::Error),

    #[error("network 服务重启失败: {0}")]
    NetworkRestartError(String),

    #[error("无效的 CIDR 格式: {0}")]
    InvalidCidrFormat(String),

    #[error("无效的 VLAN ID: {0}，有效范围 1-4094")]
    InvalidVlanId(u16),

    #[error("无效的 bonding 模式: {0}，有效范围 0-6")]
    #[allow(dead_code)]
    InvalidBondMode(u8),

    #[error("参数冲突: {0}")]
    ConflictingArguments(String),

    #[error("权限不足，需要 root 权限运行")]
    #[allow(dead_code)]
    PermissionDenied,

    #[error("用户取消操作")]
    UserCancel,

    #[error("系统缺少 systemd (需要 RHEL 7 或更高版本)")]
    MissingSystemd,
}

pub type Result<T> = std::result::Result<T, BondError>;
