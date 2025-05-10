use chrono::{DateTime, Duration, Utc};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::time::sleep;

use crate::config::{get_push_interval_enum, CONFIG};
use crate::trade_store::save_to_file;
use crate::types::TradeHistory;
use crate::telegram::{SUBSCRIBERS, send_message_to};
use crate::trade_store::get_all;

/// 获取下一个对齐的 UTC 时间点（根据推送间隔）
fn next_aligned_time(interval: Duration) -> DateTime<Utc> {
    let now = Utc::now();
    let next_ts = ((now.timestamp() / interval.num_seconds()) + 1) * interval.num_seconds();
    DateTime::<Utc>::from_timestamp(next_ts, 0).unwrap()
}

/// 启动定时器，每次对齐后执行推送任务
pub async fn start_timer_loop<F, Fut>(trade_history: TradeHistory, mut push_callback: F)
where
    F: FnMut() -> Fut + Send + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    let interval = get_push_interval_enum().to_duration();
    println!(
        "[TIMER] 推送间隔：{} 秒",
        interval.num_seconds()
    );
    tokio::spawn(async move {
        loop {
            let next = next_aligned_time(interval);
            let now = Utc::now();
            let wait_duration = (next - now).to_std().unwrap_or_else(|_| std::time::Duration::from_secs(0));

            println!(
                "[TIMER] 下一次推送将在 UTC {} 后（{} 秒）",
                next.format("%Y-%m-%d %H:%M:%S"),
                wait_duration.as_secs()
            );

            sleep(wait_duration).await;

            // 执行推送
            push_callback().await;

            // 同步备份
            save_to_file(&trade_history, &CONFIG.backup_path);
        }
    });
}
