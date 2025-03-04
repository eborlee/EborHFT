use serde::{Serialize, Deserialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeEvent {
    pub symbol: String,
    pub price: f64,
    pub quantity: f64,
    pub is_buy: bool,
    #[serde(flatten)]
    pub extra: Value, // ✅ 这里用于存储额外字段
}

fn main() {
    let binance_json = r#"
    {
        "symbol": "BTCUSDT",
        "price": 50000.5,
        "quantity": 0.1,
        "is_buy": true,
        "trade_id": 1234567, // Binance 特定字段
        "timestamp": 1648599000000
    }
    "#;

    let trade_event: TradeEvent = serde_json::from_str(binance_json).unwrap();

    println!("Trade Event: {:?}", trade_event);
    println!("Trade ID: {:?}", trade_event.extra["trade_id"]); // ✅ 可以访问 trade_id
}
