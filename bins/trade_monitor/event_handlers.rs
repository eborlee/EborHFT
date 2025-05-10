use crate::types::{WatchedQtySet, TradeHistory};
use crate::trade_store::insert_trade;
use event_engine::event::{EventPayload, EventType};
use event_engine::event_dispatcher::EventDispatcher;
use event_engine::event_dispatcher::AsyncQueueEventDispatcher;
use chrono::{DateTime, Utc, TimeZone};

pub fn register_handlers(
    dispatcher: &mut AsyncQueueEventDispatcher,
    watched_qty: WatchedQtySet,
    trade_history: TradeHistory,
) {
    let watched_qty = watched_qty.clone();
    println!("注册聚合成交事件处理器");
    dispatcher.register(EventType::AggTrade, Box::new(move |event| {
        // println!("[聚合成交] 处理事件: {:?}\n", event);
        let EventPayload::AggTrade(trade) = &event.data else {
            return;
        };
        // println!("[聚合成交] 处理事件: {:?}\n", event);
        let symbol = trade.symbol.to_lowercase();
        let qty = &trade.quantity;

        let ts_millis_u64: u64 = trade.trade_time;
        let ts_millis_i64 = ts_millis_u64 as i64;

        let dt: DateTime<Utc> = Utc.timestamp_millis_opt(ts_millis_i64).unwrap();
        let formatted = dt.format("%Y-%m-%d %H:%M:%S%.3f").to_string();

        let map = watched_qty.read().unwrap();
        if let Some(qset) = map.get(&symbol) {
            // println!("🔍 Looking for qty = {:?} in set {:?}", qty, qset);
            if !qset.contains(qty) {
                // println!("Trade quantity {} for {} is not in the watched set", qty, symbol);
                return;
            }

        } else {
            return;
        }
        println!("[监控命中] {} 触发观察币种 {} 的观察交易数量 {}, 方向 {} ", formatted ,symbol, qty, 
            if trade.is_buyer_maker { "买" } else { "卖" });
        insert_trade(&trade_history, &symbol, qty, trade.clone());


    }));

}
