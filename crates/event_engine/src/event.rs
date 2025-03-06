use serde::{Serialize, Deserialize}; // 允许序列化和反序列化，以便于在网络中传输
use serde_json::Value; // 这里引入 `Value`
use std::collections::HashMap; // 这里引入 `HashMap`


#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub enum EventType {
    AggTrade,
    Depth,
    Kline,
    Trade
}
#[derive(Debug, Serialize, Deserialize)]
pub enum EventPayload {
    AggTrade(AggTradeEvent),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "e")]  // 根据 JSON 中 "e" 字段来区分不同事件
pub enum BinanceEvent {
    #[serde(rename = "aggTrade")]
    AggTrade(AggTradeEvent),
    // 如果将来有其他事件类型，可以在这里添加，例如：
    // #[serde(rename = "depth")]
    // Depth(DepthEvent),
    // #[serde(rename = "kline")]
    // Kline(KlineEvent),
}

impl BinanceEvent {
    pub fn event_type(&self) -> EventType {
        match self {
            BinanceEvent::AggTrade(_) => EventType::AggTrade,
            // BinanceEvent::Depth(_) => EventType::Depth,
            // BinanceEvent::Kline(_) => EventType::Kline,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AggTradeEvent {
    #[serde(alias = "e", alias = "event", default)]
    pub event: String,
    #[serde(alias = "E", alias = "eventTime", default)]
    pub event_time: u64,
    #[serde(alias = "a", alias = "aggTradeId", default)]
    pub agg_trade_id: u64,
    #[serde(alias = "s", alias = "symbol", default)]
    pub symbol: String,
    #[serde(alias = "p", alias = "price", default)]
    pub price: String,
    #[serde(alias = "q", alias = "quantity", default)]
    pub quantity: String,
    #[serde(alias = "T", alias = "tradeTime", default)]
    pub trade_time: u64,
    #[serde(alias = "m", alias = "isBuyerMaker", default)]
    pub is_buyer_maker: bool,

    #[serde(skip)]  // ✅ 这个字段不会被 `serde_json` 解析
    pub received_timestamp: u128, // ✅ 记录 WebSocket 接收到的时间戳

    // 捕获额外的未知字段
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct OrderBookEvent {
//     pub symbol: String,
//     pub bids: Vec<(f64, f64)>,
//     pub asks: Vec<(f64, f64)>, // 价格，数量
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct TradeEvent {
//     pub symbol: String,
//     pub price: f64,
//     pub quantity: f64,
//     pub side: String,
// }