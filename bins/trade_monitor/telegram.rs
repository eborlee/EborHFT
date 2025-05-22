// src/telegram.rs
use chrono::Utc;

use crate::config::CONFIG;
use reqwest::Client;
use std::sync::RwLock;
use once_cell::sync::Lazy;
use crate::trade_store::get_by_symbol_qty;
use crate::trade_store::get_all;
use crate::types::{TradeHistory, WatchedQtySet};
use crate::indicators::{compute_symbol_imbalance_series,summarize_imbalance_series};
use teloxide::types::{BotCommand};
use chrono::{DateTime, Duration, TimeZone};

// bins/trade_monitor/telegram.rs 顶部添加：
use teloxide::prelude::*; // 确保引入所有必要类型（尤其是 `Message`）
use teloxide::utils::command::BotCommands;
use std::collections::{HashMap, HashSet, VecDeque};
use event_engine::event::AggTradeEvent;

use teloxide::Bot;
use teloxide::types::ParseMode;
use chrono::NaiveDateTime;
use teloxide::requests::RequesterExt;

use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};

const SUBSCRIBERS_PATH: &str = "subscribers.json";

pub static SUBSCRIBERS: Lazy<RwLock<HashSet<String>>> = Lazy::new(|| {
    let set = load_subscribers_from_file().unwrap_or_default();
    RwLock::new(set)
});

pub fn save_subscribers_to_file() {
    let guard = SUBSCRIBERS.read().unwrap();
    let json = serde_json::to_string_pretty(&*guard).unwrap();
    fs::write(SUBSCRIBERS_PATH, json).unwrap();
}

fn load_subscribers_from_file() -> Option<HashSet<String>> {
    if !Path::new(SUBSCRIBERS_PATH).exists() {
        return None;
    }

    let content = fs::read_to_string(SUBSCRIBERS_PATH).ok()?;
    serde_json::from_str(&content).ok()
}


pub async fn send_message_to(chat_id: &str, text: &str) {
    let url = format!("https://api.telegram.org/bot{}/sendMessage", CONFIG.telegram.token);

    let client = Client::new();
    let _ = client
        .post(url)
        .form(&[
            ("chat_id", chat_id),
            ("text", text),
            ("parse_mode", "Markdown"),
        ])
        .send()
        .await;
}

// pub async fn broadcast_message(text: &str) {
//     for id in &CONFIG.telegram.allowed_chat_ids {
//         let _ = send_message_to(id, text).await;
//     }
// }


/// 启动 bot 接收消息（需单独线程运行）
pub async fn start_bot(trade_history: TradeHistory, watched_qty: WatchedQtySet) {
    let bot = Bot::new(&CONFIG.telegram.token);
    // 注册命令显示到输入框左侧按钮中
    let commands = vec![
        BotCommand::new("start", "开始使用"),
        BotCommand::new("subscribe", "订阅推送"),
        BotCommand::new("unsubscribe", "取消订阅"),
        BotCommand::new("list", "查看当前监控对象"),
        BotCommand::new("imbalance", "【主要】查看偏移"),
        BotCommand::new("status", "查看缓存统计"),
        // BotCommand::new("detail", "查询某币种成交明细"),
    ];

    if let Err(e) = bot.set_my_commands(commands).await {
        eprintln!("设置命令失败: {:?}", e);
    }


    teloxide::repl(bot.clone(), move |message: Message| {
        let bot = bot.clone(); // 显式 clone 保持 `Fn`
        let trade_history = trade_history.clone();
        let watched_qty = watched_qty.clone();

        async move {
            let text = message.text().unwrap_or("").trim();
            let sender_id = message.chat.id.to_string();
            let bar_interval = Duration::minutes(15);

            match text {
                "/start" => {
                    let help_msg = "\
👋 欢迎!
时间 中央时区 UTC

/subscribe 开启定时推送
/unsubscribe 取消订阅

其他指令:
/list 本 bot 当前监控币种及数量
/imbalance 所有监控品种资金偏移统计
/status 查看缓存中的成交数据条数
/detail SYMBOL Q 查询某币种某数量的成交明细，如 `/detail btcusdt 10.023`
";
                    bot.send_message(sender_id, help_msg).send().await?;
                }
                "/status" => {
                    let cloned = {
                        let lock = trade_history.lock().unwrap();
                        lock.clone()
                    };
                    let summary = format_summary_snapshot(&cloned);
                    bot.send_message(sender_id, summary)
                        // .parse_mode(ParseMode::MarkdownV2)
                        .send()
                        .await?;
                }

                "/list" => {
                    // ✅ 在任何 await 之前提前读锁并 clone
                    let list_snapshot = {
                        let list = watched_qty.read().unwrap();
                        list.clone()
                    };

                    if list_snapshot.is_empty() {
                        bot.send_message(sender_id, "📭 当前没有关注任何数量")
                            .send()
                            .await?;
                    } else {
                        let mut lines = vec!["📌 当前关注数量：".to_string()];
                        for (symbol, set) in &list_snapshot {
                            if set.is_empty() {
                                continue;
                            }
                            let qtys = set.iter().cloned().collect::<Vec<_>>().join(", ");
                            lines.push(format!("• {}: [{}]", symbol.to_uppercase(), qtys));
                        }
                        let msg = lines.join("\n");
                        bot.send_message(sender_id, msg).send().await?;
                    }
                }



                "/subscribe" => {
                    let should_save = {
                    let mut subs = SUBSCRIBERS.write().unwrap();
                    if subs.insert(sender_id.clone()) {
                        println!("用户 {} 订阅成功", sender_id);
                        true
                    } else {
                        false
                    }
                }; // ✅ 写锁在这里释放

                if should_save {
                    save_subscribers_to_file(); // ✅ 此时无锁，可以安心读
                    println!("已更新订阅者列表到文件");
                }

                bot.send_message(sender_id, "✅ 已订阅！将会收到定时推送").send().await?;
                }

                "/unsubscribe" => {
                    let should_save = {
                        let mut subs = SUBSCRIBERS.write().unwrap();
                        subs.remove(&sender_id)
                    };
                    if should_save {
                        save_subscribers_to_file();
                    }

                    bot.send_message(sender_id, "🚫 已取消订阅").send().await?;
                }


                cmd if cmd.starts_with("/imbalance ") => {
                    let symbol = cmd["/imbalance ".len()..].trim().to_lowercase();

                    // 获取数据快照与 qty 集
                    let snapshot = {
                        let lock = trade_history.lock().unwrap();
                        lock.clone()
                    };
                    let watched = watched_qty.read().unwrap().clone();
                    let aligned_now = Utc
                        .timestamp_opt(
                            (Utc::now().timestamp() / bar_interval.num_seconds()) * bar_interval.num_seconds(),
                            0,
                        )
                        .single()
                        .unwrap_or_else(Utc::now);
                    if let Some(series) = compute_symbol_imbalance_series(
                        &snapshot,
                        &watched,
                        chrono::Duration::minutes(15),
                        chrono::Duration::days(3),
                    ).get(&symbol) {
                        let (v15, h1, h4, d1, d3) = summarize_imbalance_series(series, aligned_now, chrono::Duration::minutes(15));

                        let symbol_fmt = symbol.to_uppercase().replace('_', "\\_"); // MarkdownV2 转义
                        let msg = format!(
                            "📊 *{}* 资金偏移统计：\n\
                            UTC 时间：{}\n\
                            - 最新15min：{:+.3}\n\
                            - 1小时累计：{:+.3}\n\
                            - 4小时累计：{:+.3}\n\
                            - 1日累计：{:+.3}\n\
                            - 3日累计：{:+.3}",
                            symbol_fmt, aligned_now,v15, h1, h4, d1, d3
                        );

                        bot.send_message(sender_id, msg)
                            // .parse_mode(ParseMode::MarkdownV2)
                            .send()
                            .await?;
                    } else {
                        bot.send_message(sender_id, format!("⚠️ 无法找到 {} 的监控数据", symbol))
                            .send()
                            .await?;
                    }
            }

            cmd if cmd.trim() == "/imbalance" => {
                // 获取快照 + 关注集合
                let snapshot = {
                    let lock = trade_history.lock().unwrap();
                    lock.clone()
                };
                let watched = watched_qty.read().unwrap().clone();
                let aligned_now = Utc
                        .timestamp_opt(
                            (Utc::now().timestamp() / bar_interval.num_seconds()) * bar_interval.num_seconds(),
                            0,
                        )
                        .single()
                        .unwrap_or_else(Utc::now);
                let imbalance = compute_symbol_imbalance_series(
                    &snapshot,
                    &watched,
                    chrono::Duration::minutes(15),
                    chrono::Duration::days(3),
                );

                if imbalance.is_empty() {
                    bot.send_message(sender_id, "⚠️ 当前无任何监控数据").send().await?;
                } else {
                    let mut lines = vec!["📊 所有监控品种资金偏移统计：".to_string()];
                    lines.push(format!("\nUTC 时间：{}", aligned_now));
                    for (symbol, series) in imbalance {
                        let (v15, h1, h4, d1, d3) = summarize_imbalance_series(&series, aligned_now,chrono::Duration::minutes(15));

                        let line = format!(
                            "*{}*\n- 15min: {:+.3} | 1h: {:+.3} | 4h: {:+.3} | 1d: {:+.3} | 3d: {:+.3}",
                            symbol.to_uppercase().replace('_', "\\_"),
                            v15, h1, h4, d1, d3
                        );
                        lines.push(line);
                    }

                    let msg = lines.join("\n\n");

                    bot.send_message(sender_id, msg)
                        // .parse_mode(ParseMode::MarkdownV2)
                        .send()
                        .await?;
                }
            }




                cmd if cmd.starts_with("/detail ") => {
                    println!("收到 /detail 指令: {}", cmd);
                    let args: Vec<&str> = cmd.strip_prefix("/detail").unwrap().trim().split_whitespace().collect();

                    if args.len() != 2 {
                        bot.send_message(sender_id, "❌ 格式错误，应为：`/detail btcusdt 5.023`")
                            .parse_mode(ParseMode::MarkdownV2)
                            .send()
                            .await?;
                        return Ok(());
                    }

                    let symbol = args[0].to_lowercase();
                    let qty_str = args[1].trim(); // ✅ 保留原始字符串

                    let snapshot = {
                        let lock = trade_history.lock().unwrap();
                        lock.clone()
                    };
                    println!("当前快照：{:?}", snapshot);
                    let detail = format_detail_snapshot(&snapshot, &symbol, qty_str);
                    println!("用户指定数量的成交明细：{}", detail);
                    bot.send_message(sender_id, detail)
                        // .parse_mode(ParseMode::MarkdownV2)
                        .send()
                        .await?;

                }


                cmd if cmd.starts_with("/add ") => {
                    let parts: Vec<&str> = cmd["/add ".len()..].trim().split_whitespace().collect();
                    if parts.len() != 2 {
                        bot.send_message(sender_id, "❌ 格式错误，应为 `/add <symbol> <quantity>`").send().await?;
                    } else {
                        let symbol = parts[0].to_lowercase();
                        let qty_str = parts[1].trim();

                        if !qty_str.chars().all(|c| c.is_ascii_digit() || c == '.') {
                            bot.send_message(sender_id, "❌ 数量格式非法，应为纯数字或小数").send().await?;
                            return Ok(());
                        }

                        {
                            let mut qty_map = watched_qty.write().unwrap();
                            let entry = qty_map.entry(symbol.clone()).or_default();
                            entry.insert(qty_str.to_string());
                        }

                        bot.send_message(sender_id, format!("✅ 已添加 {symbol} 的关注数量 {qty_str}"))
                            .send()
                            .await?;
                    }
                }

                cmd if cmd.starts_with("/remove ") => {
                    let parts: Vec<&str> = cmd["/remove ".len()..].trim().split_whitespace().collect();
                    if parts.len() != 2 {
                        bot.send_message(sender_id, "❌ 格式错误，应为 `/remove <symbol> <quantity>`").send().await?;
                    } else {
                        let symbol = parts[0].to_lowercase();
                        let qty_str = parts[1].trim();

                        if !qty_str.chars().all(|c| c.is_ascii_digit() || c == '.') {
                            bot.send_message(sender_id, "❌ 数量格式非法，应为纯数字或小数").send().await?;
                            return Ok(());
                        }

                        {
                            let mut qty_map = watched_qty.write().unwrap();
                            if let Some(set) = qty_map.get_mut(&symbol) {
                                set.remove(qty_str);
                            }
                        }

                        bot.send_message(sender_id, format!("✅ 已移除 {symbol} 的关注数量 {qty_str}"))
                            .send()
                            .await?;
                    }
                }




                _ => {
                    bot.send_message(sender_id, "🤖 支持命令：\n\
                                 /status 查看整体状态\n\
                                 /list 查看当前关注数量\n\
                                 /btc <数量> 查看某个数量的成交明细\n\
                                 /add <数量> 添加关注数量\n\
                                 /remove <数量> 移除关注数量")
                        .send()
                        .await?;
                }
            }

            Ok(())
        }
    })
    .await;
}

/// 简要统计 summary
fn format_summary(history: &TradeHistory) -> String {
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

/// 查看指定数量明细
fn format_detail(history: &TradeHistory, symbol: &str, qty: f64) -> String {
    let key = format!("{:.3}", qty);
    match get_by_symbol_qty(history, symbol, qty) {
        Some(list) => {
            if list.is_empty() {
                return format!("⚠️ `{}` 下尚未捕获数量 {:.3} 的记录", symbol, qty);
            }
            let mut lines = vec![format!("📌 `{}` 最近 {:.3} 的成交（最多10条）", symbol, qty)];
            for t in list.iter().rev().take(10) {
                let ts = NaiveDateTime::from_timestamp_opt((t.event_time / 1000) as i64, 0)
                    .map(|dt| dt.format("%H:%M:%S").to_string())
                    .unwrap_or_else(|| t.event_time.to_string());
                lines.push(format!(
                    "- {} | {:.2} | {}",
                    ts,
                    t.price,
                    if t.is_buyer_maker { "卖单" } else { "买单" }
                ));
            }
            lines.join("\n")
        }
        None => format!("⚠️ `{}` 下无此数量 {:.3} 的记录", symbol, qty),
    }
}


fn format_summary_snapshot(
    history: &HashMap<String, HashMap<String, VecDeque<AggTradeEvent>>>,
) -> String {
    // 复制 format_summary() 原来的逻辑，但不再 lock()
    // 这里只是你自己控制的数据 snapshot，可以直接遍历
    let mut lines = vec!["📊 当前行情摘要：".to_string()];
    for (symbol, qty_map) in history {
        lines.push(format!("🔸 {}", symbol.to_uppercase()));
        for (qty, trades) in qty_map {
            lines.push(format!(
                "  - {}: {} 条记录",
                qty,
                trades.len()
            ));
        }
    }
    lines.join("\n")
}

fn format_detail_snapshot(
    history: &HashMap<String, HashMap<String, VecDeque<AggTradeEvent>>>,
    symbol: &str,
    qty: &str,
) -> String {
    let symbol_map = history.get(symbol);
    let qty_key = qty.to_string(); // 保留原始字符串
    // let qty_key = format!("{:.3}", qty);

    if let Some(qty_map) = symbol_map {
        if let Some(trades) = qty_map.get(&qty_key) {
            if trades.is_empty() {
                return format!("{} @ {}: 无成交记录", symbol, qty_key);
            }
            let mut lines = vec![format!("📄 {} @ {} 最近成交（最多30条）：", symbol, qty)];
            // 正序输出最近 30 条
            let last_trades = trades.iter().rev().take(30).collect::<Vec<_>>();
            for trade in last_trades.into_iter().rev() {
                let ts = chrono::NaiveDateTime::from_timestamp_millis(trade.event_time as i64)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S%.3f").to_string())
                    .unwrap_or_else(|| trade.event_time.to_string());

                lines.push(format!(
                    "- {} | 价格：{} | 方向：{}",
                    ts,
                    trade.price,
                    if trade.is_buyer_maker { "SELL" } else { "BUY" }
                ));
            }



            return lines.join("\n");
        }
    }

    format!("{} @ {}: 无记录", symbol, qty_key)
}
