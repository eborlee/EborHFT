use chrono::{DateTime, Duration, Utc, TimeZone};
use std::collections::{HashMap, VecDeque};
use event_engine::event::AggTradeEvent;
use std::collections::HashSet;

/// symbol -> [(bar_start_time, 平均强度)]
pub fn compute_symbol_imbalance_series(
    snapshot: &HashMap<String, HashMap<String, VecDeque<AggTradeEvent>>>,
    watched_qty: &HashMap<String, HashSet<String>>,
    bar_interval: Duration,         // 通常为 Duration::minutes(15)
    max_lookback: Duration          // 通常为 Duration::days(3)
) -> HashMap<String, Vec<(DateTime<Utc>, f64)>> {
    // 每个 symbol -> 每个 bar_time -> 累积方向加权数量
    let mut result: HashMap<String, HashMap<DateTime<Utc>, f64>> = HashMap::new();
    let now = Utc::now();
    let start_cutoff = now - max_lookback;

    for (symbol, qty_map) in snapshot {
        let Some(qty_set) = watched_qty.get(symbol) else {
            continue;
        };

        for (qty_str, trades) in qty_map {
            if !qty_set.contains(qty_str) {
                continue;
            }

            for trade in trades {
                let Some(timestamp) = Utc.timestamp_millis_opt(trade.trade_time as i64).single() else {
                    continue;
                };

                if timestamp < start_cutoff {
                    continue;
                }

                // 对齐到 bar 起始时间
                let since_epoch = timestamp.timestamp();
                let Some(bar_start) = Utc
                    .timestamp_opt(
                        (since_epoch / bar_interval.num_seconds()) * bar_interval.num_seconds(),
                        0,
                    )
                    .single()
                else {
                    continue;
                };

                // 解析真实数量
                let qty: f64 = match trade.quantity.parse() {
                    Ok(val) => val,
                    Err(_) => continue,
                };

                // 买入为正，卖出为负
                let signed_qty = if trade.is_buyer_maker { -qty } else { qty };

                result
                    .entry(symbol.clone())
                    .or_default()
                    .entry(bar_start)
                    .and_modify(|v| *v += signed_qty)
                    .or_insert(signed_qty);
            }
        }
    }

    // 排序输出：symbol -> [(bar_time, net_qty)]
    let mut series_result = HashMap::new();

    for (symbol, bar_map) in result {
        let mut sorted: Vec<_> = bar_map.into_iter().collect();
        sorted.sort_by_key(|(dt, _)| *dt);
        series_result.insert(symbol, sorted);
    }

    series_result
}


pub fn summarize_imbalance_series(
    series: &[(DateTime<Utc>, f64)],
    now: DateTime<Utc>,
    bar_interval: Duration, 
) -> (f64, f64, f64, f64, f64) {
    let mut last_15 = 0.0;
    let mut sum_1h = 0.0;
    let mut sum_4h = 0.0;
    let mut sum_1d = 0.0;
    let mut sum_3d = 0.0;

    

    // println!("aligned_now: {}", aligned_now);
    for (ts, val) in series.iter().rev() {
        let delta = now.signed_duration_since(*ts);
        if delta <= Duration::minutes(15) {
            last_15 = *val;
        }
        if delta <= Duration::hours(1) {
            sum_1h += val;
        }
        if delta <= Duration::hours(4) {
            sum_4h += val;
        }
        if delta <= Duration::days(1) {
            sum_1d += val;
        }
        if delta <= Duration::days(3) {
            sum_3d += val;
        } else {
            break;
        }
    }

    (last_15, sum_1h, sum_4h, sum_1d, sum_3d)
}
