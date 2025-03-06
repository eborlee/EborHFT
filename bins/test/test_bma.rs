// main.rs


use event_engine::event::EventType;
use event_engine::event_dispatcher::{AsyncQueueEventDispatcher, EventDispatcher};
use market_agent::market_agent::MarketAgent;
use market_agent::binance_market_agent::BinanceMarketAgent;
use feeder::websocket::WebSocket;
use feeder::websocket::BinanceWebSocketClient;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use chrono::Local;

// 以下函数用于获取时间戳
fn get_timestamp() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
}
fn get_timestamp_ms() -> String {
    let now = Local::now();
    let datetime_str = now.format("%Y-%m-%d %H:%M:%S").to_string();
    let millis = now.timestamp_subsec_millis();
    format!("{} - {}", datetime_str, millis)
}

#[tokio::main]
async fn main() {
    // 创建一个容量为 200 的异步事件分发器
    let mut async_dispatcher = AsyncQueueEventDispatcher::new(200);
    
    // 注册一个回调到 Depth 事件：简单打印收到的深度数据
    async_dispatcher.register(EventType::Depth, Box::new(|event| {
        println!("[Callback] 收到 Depth 数据: {:?}", event.data);
    }));

    // 拆分出 Producer 和 Consumer
    let (producer, mut consumer) = async_dispatcher.split();

    // 初始化 BinanceWebSocketClient（ws 部分）
    let mut ws_client = BinanceWebSocketClient::new();
    // 建立连接并订阅 "bnbusdt@aggTrade"（示例）
    ws_client.connect(vec!["bnbusdt@aggTrade"]).await.unwrap();
    ws_client.subscribe(vec!["bnbusdt@aggTrade"]).await.unwrap();

    // 创建 BinanceMarketAgent，将 ws_client 与事件分发器的 Producer 部分传入
    let market_agent = BinanceMarketAgent::new(ws_client, producer);

    // 启动 BinanceMarketAgent（内部会注册 on_depth 回调到 ws_client 的消息回调中）
    let agent_handle = {
        let agent_clone = market_agent.clone();
        tokio::spawn(async move {
            agent_clone.start().await.unwrap();
        })
    };

    // 启动一个线程处理事件队列数据（consumer.process()）
    thread::spawn(move || {
        loop {
            consumer.process();
            // thread::sleep(Duration::from_millis(1)); // 每50ms处理一次队列
        }
    });

    agent_handle.await.unwrap();
}
