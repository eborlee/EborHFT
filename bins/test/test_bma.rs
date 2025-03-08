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

// #[tokio::main]
// fn run() {
//     // 创建一个容量为 200 的异步事件分发器
//     let mut async_dispatcher = AsyncQueueEventDispatcher::new(200);
    
//     // 注册一个回调到 Depth 事件：简单打印收到的深度数据
//     async_dispatcher.register(EventType::AggTrade, Box::new(|event| {
//         // 记录当前时间戳（事件分发时间）
//         let processed_timestamp = get_timestamp();
//         let processed_timestamp_us = get_timestamp_us();
    
//         // 提取 `received_timestamp` 和 `event_time`
//         let (received_timestamp, event_time_ms) = match &event.data {
//             EventPayload::AggTrade(trade) => (trade.received_timestamp, trade.event_time),
//         };
    
//         // 计算 WebSocket → 事件分发的系统延迟
//         let system_latency = processed_timestamp_us.saturating_sub(received_timestamp);

//         // 计算 交易所 → WebSocket 数据传输的延迟
//         let exchange_latency = received_timestamp.saturating_sub(event_time_ms as u128 * 1_000);
    
//         println!(
//             "【聚合成交】系统延迟: {} µs | 网络延迟: {} µs | {:?}",
//             system_latency, exchange_latency, event.data
//         );
//     }));
    

//     // 拆分出 Producer 和 Consumer
//     let (producer, mut consumer) = async_dispatcher.split();

//     // 初始化 BinanceWebSocketClient（ws 部分）
//     let mut ws_client = BinanceWebSocketClient::new();
//     // 建立连接并订阅 "bnbusdt@aggTrade"（示例）
//     ws_client.connect(vec!["bnbusdt@aggTrade"]).await.unwrap();
//     ws_client.subscribe(vec!["bnbusdt@aggTrade"]).await.unwrap();

//     // 创建 BinanceMarketAgent，将 ws_client 与事件分发器的 Producer 部分传入
//     // let market_agent = BinanceMarketAgent::new(ws_client, producer);

//     // // 启动 BinanceMarketAgent（内部会注册 on_depth 回调到 ws_client 的消息回调中）
//     // let agent_handle = {
//     //     let agent_clone = market_agent.clone();
//     //     tokio::spawn(async move {
//     //         agent_clone.start().await.unwrap();
//     //     })
//     // };
//     let market_agent = Arc::new(BinanceMarketAgent::new(ws_client, producer)); // ✅ 用 `Arc::new()`

//     let agent_handle = tokio::spawn({
//         let market_agent = Arc::clone(&market_agent);
//         async move {
//             market_agent.start().await.unwrap();
//         }
//     });

//     #[cfg(feature = "set_affinity")]
//     {
//         let core_ids = affinity::get_core_ids().unwrap();
//         if let Some(core_id) = core_ids.first() {
//             affinity::set_thread_affinity([*core_id]).unwrap();
//         }
//     }

//     // 启动一个线程处理事件队列数据（consumer.process()）
//     thread::spawn(move || {
//         loop {
//             consumer.process();
//             // thread::sleep(Duration::from_millis(1)); // 每50ms处理一次队列
//         }
//     });

//     // agent_handle.await.unwrap();
//     agent_handle.await.unwrap();
    
// }


fn main() {
    // 指定线程池数量为4个线程
    let rt = Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async {
        let mut async_dispatcher = AsyncQueueEventDispatcher::new(200);
    
    // 注册一个回调到 Depth 事件：简单打印收到的深度数据
    async_dispatcher.register(EventType::AggTrade, Box::new(|event| {
        // 记录当前时间戳（事件分发时间）
        let processed_timestamp = get_timestamp();
        let processed_timestamp_us = get_timestamp_us();
    
        // 提取 `received_timestamp` 和 `event_time`
        let (received_timestamp, event_time_ms) = match &event.data {
            EventPayload::AggTrade(trade) => (trade.received_timestamp, trade.event_time),
        };
    
        // 计算 WebSocket → 事件分发的系统延迟
        let system_latency = processed_timestamp_us.saturating_sub(received_timestamp);

        // 计算 交易所 → WebSocket 数据传输的延迟
        let exchange_latency = received_timestamp.saturating_sub(event_time_ms as u128 * 1_000);
    
        println!(
            "【聚合成交】系统延迟: {} µs | 网络延迟: {} µs | {:?}",
            system_latency, exchange_latency, event.data
        );
    }));
    

    // 拆分出 Producer 和 Consumer
    let (producer, mut consumer) = async_dispatcher.split();

    // 初始化 BinanceWebSocketClient（ws 部分）
    let mut ws_client = BinanceWebSocketClient::new();
    // 建立连接并订阅 "bnbusdt@aggTrade"（示例）
    ws_client.connect(vec!["bnbusdt@aggTrade"]).await.unwrap();
    ws_client.subscribe(vec!["bnbusdt@aggTrade"]).await.unwrap();

    // 创建 BinanceMarketAgent，将 ws_client 与事件分发器的 Producer 部分传入
    // let market_agent = BinanceMarketAgent::new(ws_client, producer);

    // // 启动 BinanceMarketAgent（内部会注册 on_depth 回调到 ws_client 的消息回调中）
    // let agent_handle = {
    //     let agent_clone = market_agent.clone();
    //     tokio::spawn(async move {
    //         agent_clone.start().await.unwrap();
    //     })
    // };
    let market_agent = Arc::new(BinanceMarketAgent::new(ws_client, producer)); // ✅ 用 `Arc::new()`

    let agent_handle = tokio::spawn({
        let market_agent = Arc::clone(&market_agent);
        async move {
            market_agent.start().await.unwrap();
        }
    });

    #[cfg(feature = "set_affinity")]
    {
        let core_ids = affinity::get_core_ids().unwrap();
        if let Some(core_id) = core_ids.first() {
            affinity::set_thread_affinity([*core_id]).unwrap();
        }
    }

    // 启动一个线程处理事件队列数据（consumer.process()）
    thread::spawn(move || {
        loop {
            consumer.process();
            // thread::sleep(Duration::from_millis(1)); // 每50ms处理一次队列
        }
    });

    // agent_handle.await.unwrap();
    agent_handle.await.unwrap();
    
    });
}