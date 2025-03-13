/// 测试整合事件引擎和订单簿引擎

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use chrono::Local;
use tokio::runtime::Runtime;

use event_engine::event::{EventType, EventPayload, DepthEvent};
use event_engine::event_dispatcher::{AsyncQueueEventDispatcher, EventDispatcher, EventData};

use market_agent::market_agent::MarketAgent;
use market_agent::binance_market_agent::BinanceMarketAgent;

use feeder::websocket::WebSocket;
use feeder::websocket::BinanceWebSocketClient;
use orderbook::engine::OrderBookEngine;

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
        .as_micros()
}

fn get_timestamp_ms() -> String {
    let now = Local::now();
    let datetime_str = now.format("%Y-%m-%d %H:%M:%S").to_string();
    let millis = now.timestamp_subsec_millis();
    format!("{} - {}", datetime_str, millis)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建 dispatcher，容量为 200
    let mut async_dispatcher = AsyncQueueEventDispatcher::new(200);

    // 创建一个共享的订单簿引擎实例，针对 "BTCUSDT"
    let orderbook_engine = Arc::new(Mutex::new(OrderBookEngine::new("BTCUSDT")));
    // 注册 Depth 事件的回调：dispatcher 收到深度事件后调用订单簿的 push_update
    {
        let engine_clone = Arc::clone(&orderbook_engine);
        async_dispatcher.register(EventType::Depth, Box::new(move |event: &EventData| {
            // 锁定订单簿引擎并调用 push_update，将 event 数据传入
            let mut engine = engine_clone.lock().unwrap();
            // println!("收到深度事件: {:?}", event);
            if let Err(e) = engine.push_update(event.clone()) {
                eprintln!("订单簿更新失败: {}", e);
            }
        }));
    }

    // 注册回调，每次订单簿更新时打印最佳买卖价
    {
        let mut engine = orderbook_engine.lock().unwrap();

        engine.register_callback(|order_book| {
            let spread = order_book.best_bid()
                .zip(order_book.best_ask())
                .map(|((bid_price, _), (ask_price, _))| bid_price - ask_price);
            println!(
                "回调：最佳买价: {:?}, 最佳卖价: {:?}, 最佳买卖价差: {:?}, bids档位: {:?}, asks档位: {:?}",
                order_book.best_bid(),
                order_book.best_ask(),
                spread,
                order_book.bid_levels(),
                order_book.ask_levels(),
            );
        });
    }
    

    // 从 dispatcher 分离出生产者和消费者
    let (producer, mut consumer) = async_dispatcher.split();

    // 创建并连接 Binance WebSocket 客户端，订阅 BTCUSDT 的 depth 事件
    let mut ws_client = BinanceWebSocketClient::new();
    ws_client.connect(vec!["btcusdt@depth@100ms"]).await?;
    ws_client.subscribe(vec!["btcusdt@depth@100ms"]).await?;

    // 创建 market agent，将 ws_client 和 dispatcher 的 producer 传入
    let mut market_agent = BinanceMarketAgent::new(ws_client, producer);

    // 开启一个新线程运行 market_agent（异步调用）
    thread::spawn(move || {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            market_agent.start().await;
        });
    });

    let consumer_thread = thread::spawn(move || {
        loop {
            consumer.process();
            // std::thread::sleep(Duration::from_millis(1));
        }
    });
    // thread::sleep(Duration::from_secs(5));
    // 初始化订单簿：通过 REST 接口获取深度快照，并填充订单簿数据
    {
        let mut engine = orderbook_engine.lock().unwrap();
        // let snapshot = engine.fetch_depth_snapshot().await?;
        // engine.last_update_id = snapshot.last_update_id;
        // engine.order_book = snapshot.to_order_book();
         // 休眠2秒
        engine.initialize().await?;
        println!("初始化订单簿成功，当前状态：");
        println!(
            "最佳20买价: {:?}, 最佳20卖价: {:?}",
            engine.order_book.top_n_bids(20),
            engine.order_book.top_n_asks(20)
        );
    }
    consumer_thread.join().unwrap();
    Ok(())
    // 主循环：不断处理 dispatcher 队列中的事件
    // loop {
    //     consumer.process();
    //     thread::sleep(Duration::from_millis(1));
    // }
}
