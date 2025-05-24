use std::collections::{HashMap, VecDeque};
use std::fs;
use std::io::Write;
use std::sync::Mutex;

use crate::config::CONFIG;
use crate::types::TradeHistory;
use event_engine::event::AggTradeEvent;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use chrono::TimeZone;

/// 向 TradeHistory 中插入一条交易记录（自动维护滑动窗口）
pub fn insert_trade(history: &TradeHistory, symbol: &str, qty: &String, trade: AggTradeEvent) {
    let key = qty.clone(); // ✅ 保留原始字符串，不做 format
    let mut guard = history.lock().unwrap();
    let entry = guard
        .entry(symbol.to_string())
        .or_insert_with(HashMap::new)
        .entry(key)
        .or_insert_with(VecDeque::new);

    entry.push_back(trade);

    if entry.len() > CONFIG.history_max_len {
        entry.pop_front();
    }
}


pub fn get_all(history: &TradeHistory) -> HashMap<String, HashMap<String, VecDeque<AggTradeEvent>>> {
    let guard = history.lock().unwrap();
    guard
        .iter()
        .map(|(symbol, qty_map)| {
            (
                symbol.clone(),
                qty_map
                    .iter()
                    .map(|(q, v)| (q.clone(), v.iter().cloned().collect::<VecDeque<_>>()))
                    .collect(),
            )
        })
        .collect()
}


pub fn get_by_symbol_qty(history: &TradeHistory, symbol: &str, qty: f64) -> Option<Vec<AggTradeEvent>> {
    let key = format!("{:.3}", qty);

    let guard = history.lock().unwrap();
    guard.get(symbol)?.get(&key).map(|v| v.iter().cloned().collect())
}


/// ✅ JSON 序列化用结构
#[derive(Serialize, Deserialize)]
pub struct SerializableHistory(pub HashMap<String, HashMap<String, Vec<AggTradeEvent>>>);


/// 将当前内存中的 TradeHistory 保存为本地文件（backup）
pub fn save_to_file(history: &TradeHistory, path: &str) {
    let map = history.lock().unwrap();
    let serializable = SerializableHistory(
        map.iter()
            .map(|(symbol, inner)| {
                let inner_map = inner
                    .iter()
                    .map(|(qty, list)| (qty.clone(), list.iter().cloned().collect()))
                    .collect();
                (symbol.clone(), inner_map)
            })
            .collect(),
    );

    if let Ok(json) = serde_json::to_string_pretty(&serializable) {
        let _ = fs::write(path, json);
        println!("[备份] 已保存到 {path}");
    } else {
        eprintln!("[备份] 序列化失败");
    }
}


pub fn load_from_file(history: &TradeHistory, path: &str) {
    match fs::read_to_string(path) {
        Ok(content) => {
            if let Ok(SerializableHistory(map)) = serde_json::from_str::<SerializableHistory>(&content) {
                let mut target = history.lock().unwrap();
                for (symbol, qty_map) in map {
                    let inner = target.entry(symbol).or_insert_with(HashMap::new);
                    for (qty_str, list) in qty_map {
                        inner.insert(qty_str, VecDeque::from(list));
                    }
                }
                println!("[恢复] 已从 {path} 加载记录");
            } else {
                eprintln!("[恢复] JSON 解析失败");
            }
        }
        Err(_) => {
            println!("[恢复] 未发现本地备份，跳过加载");
        }
    }
}



pub fn get_recent_trades(history: &TradeHistory,symbol: &str, since: DateTime<Utc>) -> Vec<AggTradeEvent> {
    let history = history.lock().unwrap(); // ✅ 显式加锁
    history.get(symbol)
        .map(|qty_map| {
            qty_map.values().flat_map(|trades| {
                trades.iter()
                    .filter(|t| Utc.timestamp_millis(t.event_time as i64) >= since)
                    .cloned()
            }).collect()
        })
        .unwrap_or_default()
}
