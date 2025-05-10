use once_cell::sync::Lazy;
use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use crate::types::WatchedQtySet;
use chrono::Duration;
use crate::types::TradeHistory;

/// 从 config.toml 中加载配置
pub static CONFIG: Lazy<Config> = Lazy::new(|| {
    let content = fs::read_to_string("config.toml").expect("读取 config.toml 失败");
    toml::from_str(&content).expect("解析 config.toml 失败")
});

#[derive(Debug, Deserialize)]
pub struct Config {
    /// 推送间隔，例如 "5min"、"1h"、"4h"
    pub push_interval: String,

    /// 每个 qty 保留多少条记录
    pub history_max_len: usize,

    /// 关注的 qty 列表
    pub watched_quantities: HashMap<String, Vec<String>>,

    /// 备份文件路径
    pub backup_path: String,

    pub telegram: TelegramConfig,
}

#[derive(Debug, Deserialize)]
pub struct TelegramConfig {
    pub token: String,
}

#[derive(Debug, Clone, Copy)]
pub enum PushInterval {
    Min5,
    Min15,
    Hour1,
    Hour4,
    Hour8,
    Day1,
}

impl PushInterval {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "5min" => Some(Self::Min5),
            "15min" => Some(Self::Min15),
            "1h" => Some(Self::Hour1),
            "4h" => Some(Self::Hour4),
            "8h" => Some(Self::Hour8),
            "1d" => Some(Self::Day1),
            _ => None,
        }
    }

    pub fn to_duration(self) -> Duration {
        match self {
            Self::Min5 => Duration::minutes(5),
            Self::Min15 => Duration::minutes(15),
            Self::Hour1 => Duration::hours(1),
            Self::Hour4 => Duration::hours(4),
            Self::Hour8 => Duration::hours(8),
            Self::Day1 => Duration::days(1),
        }
    }
}

/// 全局解析推送间隔为枚举
pub fn get_push_interval_enum() -> PushInterval {
    PushInterval::from_str(&CONFIG.push_interval).expect("无效的 push_interval 配置")
}

/// 获取 qty 集合（通常用于过滤）
pub fn get_watched_qty_set() -> WatchedQtySet {
    let mut result = HashMap::new();
    for (symbol, list) in &CONFIG.watched_quantities {
        let set: HashSet<String> = list.iter().cloned().collect();  // ✅ 不再 format
        result.insert(symbol.to_lowercase(), set);
    }
    Arc::new(RwLock::new(result))
}
