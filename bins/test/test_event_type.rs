use serde::{Serialize, Deserialize};
use serde_json::Value; // 这里引入 `Value`
use std::collections::HashMap; // 这里引入 `HashMap`


use event_engine::event;
use event_engine::event::EventType;
use event_engine::event::BinanceEvent;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 模拟 Binance 发送的 JSON
    let json_str = r#"{
        "e": "aggTrade",
        "E": 1741225971347,
        "a": 670434678,
        "s": "BNBUSDT",
        "p": "603.230",
        "q": "0.21",
        "T": 1741225971276,
        "m": true
    }"#;

    // 解析 JSON 为 BinanceEvent
    let event: BinanceEvent = serde_json::from_str(json_str)?;

    // 匹配事件类型
    match &event {
        BinanceEvent::AggTrade(data) => {
            println!("✅ 解析到 AggTrade 事件:");
            println!("交易对: {}", data.symbol);
            println!("价格: {}", data.price);
            println!("数量: {}", data.quantity);
            println!("是否买方主动: {}", data.is_buyer_maker);
            println!("原始 JSON 解析后的数据结构: {:#?}", data);
        }
    }

    // 映射到统一的事件类型
    println!("事件类型映射为: {:?}", event.event_type());

    println!("事件数据：{:?}", event);
    println!("事件数据：{:?}", event.AggTrade);

    Ok(())
}