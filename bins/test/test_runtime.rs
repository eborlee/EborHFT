// main.rs
use std::error::Error;
use std::sync::{Arc, Mutex};

use orderbook::engine::OrderBookEngine;
use app::runtime::Runtime;
use event_engine::event::{EventType};
use event_engine::event_dispatcher::EventData;
use common::exchange::Exchange;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // 创建 Runtime（核心系统）
    let mut app = Runtime::new(Exchange::Binance, 200).await?;
    app.subscribe(vec!["btcusdt@depth@100ms"]).await?;

    // 创建订单簿模块（作为独立应用层模块），这里用 Arc<Mutex<>> 包装以便跨线程共享
    let orderbook_engine = Arc::new(Mutex::new(OrderBookEngine::new("BTCUSDT")));

    // 注册订单簿模块的更新回调，打印订单簿状态
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


    // 注册订单簿的更新函数作为回调到 Runtime 中，订阅 Depth 事件
    {
        let orderbook_clone = Arc::clone(&orderbook_engine);
        app.register_event_callback(EventType::Depth, Box::new(move |event: &EventData| {
            let mut engine = orderbook_clone.lock().unwrap();
            if let Err(e) = engine.push_update(event.clone()) {
                eprintln!("订单簿更新失败: {}", e);
            }
        }));
    }

    // 初始化订单簿
    {
        let mut engine = orderbook_engine.lock().unwrap();
        engine.initialize().await?;
        println!("初始化订单簿成功，当前状态：");
        println!(
            "最佳20买价: {:?}, 最佳20卖价: {:?}",
            engine.order_book.top_n_bids(20),
            engine.order_book.top_n_asks(20)
        );
    }

    // 启动服务（市场代理、事件循环）
    app.start_service().await?;

    Ok(())


}
