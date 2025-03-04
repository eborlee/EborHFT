use serde::{Serialize, Deserialize}; // 允许序列化和反序列化，以便于在网络中传输

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub enum EventType {
    Trade,
    AggTrade,
    Depth,
    Kline,
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