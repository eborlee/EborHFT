// main.rs


use event_engine::event::EventType;
use event_engine::event::EventPayload;
use event_engine::event_dispatcher::{AsyncQueueEventDispatcher, EventDispatcher};
use market_agent::market_agent::MarketAgent;
use market_agent::binance_market_agent::BinanceMarketAgent;
use feeder::websocket::WebSocket;
use feeder::websocket::BinanceWebSocketClient;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use chrono::Local;
use tokio::runtime::Builder;
// 以下函数用于获取时间戳
fn get_timestamp() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

fn get_timestamp_us() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_micros() // ✅ 以微秒（µs）为单位
}





fn get_timestamp_ms() -> String {
    let now = Local::now();
    let datetime_str = now.format("%Y-%m-%d %H:%M:%S").to_string();
    let millis = now.timestamp_subsec_millis();
    format!("{} - {}", datetime_str, millis)
}



#[tokio::main]
async fn main() {
    let mut async_dispatcher = AsyncQueueEventDispatcher::new(200);
    
    async_dispatcher.register(EventType::AggTrade, Box::new(|event| {
        let processed_timestamp = get_timestamp_us();
        let (received_timestamp, event_time_ms) = match &event.data {
            EventPayload::AggTrade(trade) => (trade.received_timestamp, trade.event_time),
        };
        let system_latency = processed_timestamp.saturating_sub(received_timestamp);
        let exchange_latency = received_timestamp.saturating_sub(event_time_ms as u128 * 1_000);
    
        println!(
            "【聚合成交】系统延迟: {} µs | 网络延迟: {} µs | {:?}",
            system_latency, exchange_latency, event.data
        );
    }));

    let (producer, mut consumer) = async_dispatcher.split();

    let mut ws_client = BinanceWebSocketClient::new();
    ws_client.connect(vec!["bnbusdt@aggTrade"]).await.unwrap();
    ws_client.subscribe(vec!["bnbusdt@aggTrade"]).await.unwrap();

    

    let mut market_agent = BinanceMarketAgent::new(ws_client, producer);

    thread::spawn(move || {
        if let Err(e) = market_agent.start_sync() {
            eprintln!("MarketAgent 错误：{}", e);
        }
    });

    

    loop {
        consumer.process();
        // tokio::time::sleep(Duration::from_millis(1)).await;
    }
}
