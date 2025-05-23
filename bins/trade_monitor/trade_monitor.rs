mod config;
mod event_handlers;
mod timer;
mod trade_store;
mod types;
mod telegram;
mod indicators;
use crate::config::{get_watched_qty_set, CONFIG};
use crate::event_handlers::register_handlers;
use crate::timer::start_timer_loop;
use crate::trade_store::load_from_file;
use crate::types::TradeHistory;
use crate::telegram::{SUBSCRIBERS, send_message_to};
use crate::trade_store::get_all;
use crate::indicators::{compute_symbol_imbalance_series,summarize_imbalance_series};

use std::collections::VecDeque;

use feeder::websocket::WebSocket;
use feeder::websocket::BinanceWebSocketClient;
use chrono::{NaiveDateTime, TimeZone, Utc};
use event_engine::event::AggTradeEvent;

use event_engine::event::EventType;
use event_engine::event_dispatcher::AsyncQueueEventDispatcher;
use market_agent::market_agent::MarketAgent;
use market_agent::binance_market_agent::BinanceMarketAgent;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use tokio::runtime::Runtime;
use chrono::{DateTime, Duration};



/// 简单格式化最近成交摘要
fn format_trade_summary(history: &TradeHistory) -> String {


    let map = get_all(history);
    if map.is_empty() {
        return "⚠️ 当前无监控到的成交记录。".to_string();
    }

    let mut lines = vec!["📊 当前监控成交数量摘要：".to_string()];

    for (symbol, qty_map) in map {
        lines.push(format!("- `{}`", symbol));

        for (qty, trades) in qty_map {
            let count = trades.len();

            let latest_time = trades.into_iter().last()
                .map(|t| {
                    let ts = t.event_time / 1000;
                    NaiveDateTime::from_timestamp_opt(ts as i64, 0)
                        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                        .unwrap_or_else(|| format!("{}", ts))
                })
                .unwrap_or_else(|| "无".to_string());

            lines.push(format!(
                "  - 数量 `{}`：共 {} 条，最近时间：{}",
                qty, count, latest_time
            ));
        }
    }

    lines.join("\n")
}





async fn run_system() {
    println!("[启动] 加载配置...");
    let watched = get_watched_qty_set();
    let trade_history: TradeHistory = Arc::new(Mutex::new(HashMap::new()));

    // 尝试从本地恢复缓存
    load_from_file(&trade_history, &CONFIG.backup_path);

    let mut dispatcher = AsyncQueueEventDispatcher::new(500);
    register_handlers(&mut dispatcher, watched.clone(), trade_history.clone());

    let (producer, mut consumer) = dispatcher.split();

    println!("[启动] 初始化 Binance WebSocket...");
    let mut ws_client = BinanceWebSocketClient::new();
    ws_client
        .connect(vec!["btcusdt@aggTrade"])
        .await
        .unwrap();
    ws_client
        .subscribe(vec!["btcusdt@aggTrade"])
        .await
        .unwrap();

    let mut market_agent = BinanceMarketAgent::new(ws_client, producer);

    println!("[启动] 启动 MarketAgent...");
    thread::spawn(move || {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            market_agent.start().await;
        });
    });


    // ✅ 启动 Telegram Bot 监听指令
    tokio::spawn(telegram::start_bot(trade_history.clone(), watched.clone()));
    println!("[启动] 启动 Telegram Bot监听指令...");

    println!("[启动] 启动定时推送器...");
    let cloned_history = trade_history.clone();
    start_timer_loop(cloned_history.clone(), move || {
        let trade_history = cloned_history.clone();
        let watched_map = watched.read().unwrap().clone(); // ✅ 提前 clone HashMap，释放锁

        
        async move {
            let bar_interval = Duration::minutes(15);
            let snapshot = get_all(&trade_history);
            let ids = SUBSCRIBERS.read().unwrap().clone();

            let imbalance = compute_symbol_imbalance_series(
                &snapshot,
                &watched_map,
                chrono::Duration::minutes(15),
                chrono::Duration::days(3),
            );
            let aligned_now = Utc
                        .timestamp_opt(
                            (Utc::now().timestamp() / bar_interval.num_seconds()) * bar_interval.num_seconds(),
                            0,
                        )
                        .single()
                        .unwrap_or_else(Utc::now);

            for (symbol, series) in imbalance {
                let (v15, h1, h4, d1, d3) = summarize_imbalance_series(&series, aligned_now,chrono::Duration::minutes(15));

                let msg = format!(
                    "📊 *{}* 资金偏移统计：\n\
                    UTC 时间：{}\n\
                    - 最新15min：{:+.3}\n\
                    - 1小时累计：{:+.3}\n\
                    - 4小时累计：{:+.3}\n\
                    - 1日累计：{:+.3}\n\
                    - 3日累计：{:+.3}",
                    symbol.to_uppercase(),aligned_now, v15, h1, h4, d1, d3
                );

                for id in &ids {
                    send_message_to(id, &msg).await;
                }
            }
        }
    }).await;


    println!("[启动] 启动主消费循环...");
    // 启动 consumer 消费线程（阻塞）
    std::thread::spawn(move || {
        consumer.process(); // 阻塞式
    });

    // 保持主线程存活（或用 ctrl_c 等待）
    tokio::signal::ctrl_c().await.unwrap();
    println!("🛑 收到 Ctrl+C，退出程序");

    
}


#[tokio::main]
async fn main() {
    tokio::select! {
        _ = run_system() => {
            println!("✅ 系统任务正常结束");
        }
        _ = tokio::signal::ctrl_c() => {
            println!("🛑 收到 Ctrl+C，准备退出...");
        }
    }


    println!("🎯 程序已安全退出");
}
