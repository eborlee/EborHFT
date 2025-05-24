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
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum EventPayload {
    AggTrade(AggTradeEvent),
    Depth(DepthEvent),
    Kline(KlineEvent),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "e")]  // 根据 JSON 中 "e" 字段来区分不同事件
pub enum BinanceEvent {
    #[serde(rename = "aggTrade")]
    AggTrade(AggTradeEvent),
    // 如果将来有其他事件类型，可以在这里添加，例如：
    #[serde(rename = "depthUpdate")]
    Depth(DepthEvent),
    // #[serde(rename = "kline")]
    // Kline(KlineEvent),
    #[serde(rename = "continuous_kline")]
    Kline(KlineEvent),
}

impl BinanceEvent {
    pub fn event_type(&self) -> EventType {
        match self {
            BinanceEvent::AggTrade(_) => EventType::AggTrade,
            BinanceEvent::Depth(_) => EventType::Depth,
            BinanceEvent::Kline(_) => EventType::Kline,
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DepthEvent {
    #[serde(alias = "e", alias = "event", default)]
    pub event: String,               // 事件类型

    #[serde(alias = "E", alias = "eventTime", default)]
    pub event_time: u64,             // 事件时间

    #[serde(alias = "T", alias = "tradeTime", default)]
    pub trade_time: u64,             // 交易时间

    #[serde(alias = "s", alias = "symbol", default)]
    pub symbol: String,              // 交易对

    #[serde(alias = "U", default)]
    pub first_update_id: u64,        // 从上次推送至今新增的第一个 update Id

    #[serde(alias = "u", default)]
    pub last_update_id: u64,         // 从上次推送至今新增的最后一个 update Id

    #[serde(alias = "pu", default)]
    pub previous_update_id: u64,     // 上次推送的最后一个 update Id（上条消息的 u 字段）

    #[serde(alias = "b", default)]
    pub bids: Vec<(String, String)>, // 买方档位，每个元素为 (价格, 数量)

    #[serde(alias = "a", default)]
    pub asks: Vec<(String, String)>, // 卖方档位，每个元素为 (价格, 数量)

    #[serde(skip)]
    pub received_timestamp: u128,    // 记录 WebSocket 接收到的时间戳

    #[serde(flatten)]
    pub extra: HashMap<String, Value>, // 捕获额外的未知字段
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


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KlineInner {
    #[serde(alias = "t")]
    pub start_time: u64,
    #[serde(alias = "T")]
    pub end_time: u64,
    #[serde(alias = "i")]
    pub interval: String,
    #[serde(alias = "f")]
    pub first_trade_id: u64,
    #[serde(alias = "L")]
    pub last_trade_id: u64,
    #[serde(alias = "o")]
    pub open: String,
    #[serde(alias = "c")]
    pub close: String,
    #[serde(alias = "h")]
    pub high: String,
    #[serde(alias = "l")]
    pub low: String,
    #[serde(alias = "v")]
    pub volume: String,
    #[serde(alias = "n")]
    pub trade_count: u64,
    #[serde(alias = "x")]
    pub is_final: bool,
    #[serde(alias = "q")]
    pub quote_asset_volume: String,
    #[serde(alias = "V")]
    pub taker_buy_base_volume: String,
    #[serde(alias = "Q")]
    pub taker_buy_quote_volume: String,
    #[serde(alias = "B")]
    pub ignore: String
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KlineEvent {
    #[serde(alias = "e", default)]
    pub event: String,
    #[serde(alias = "E")]
    pub event_time: u64,
    #[serde(alias = "ps")]
    pub pair: String,             // ✅ 保留这个
    #[serde(alias = "ct")]
    pub contract_type: String,    // ✅ 合约类型，必须有
    #[serde(alias = "k")]
    pub kline: KlineInner,

    #[serde(skip)]
    pub received_timestamp: u128,
}