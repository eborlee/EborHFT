// common/exchange.rs



/// 定义支持的交易所枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Exchange {
    Binance,
}

impl Exchange {
    /// 将枚举值转换为对应的字符串标识
    pub fn as_str(&self) -> &'static str {
        match self {
            Exchange::Binance => "binance",
        }
    }
}

