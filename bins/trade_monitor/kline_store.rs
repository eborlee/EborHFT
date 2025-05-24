use std::fs::OpenOptions;
use std::io::Write;
use chrono::NaiveDateTime;
use event_engine::event::KlineEvent;

use chrono::{DateTime, Utc, Datelike};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use chrono::TimeZone;


pub fn save_kline_to_file(symbol: &str, kline: &KlineEvent) {
    use std::fs::{create_dir_all, OpenOptions};
    use std::io::Write;
    use chrono::NaiveDateTime;

    let ts = kline.kline.start_time / 1000;
    let dt = NaiveDateTime::from_timestamp_opt(ts as i64, 0)
        .unwrap_or_else(|| NaiveDateTime::from_timestamp(0, 0));
    let month_str = dt.format("%Y%m").to_string(); // e.g., "202505"

    let interval = kline.kline.interval.to_lowercase(); // 15m、1h 等

    let folder = "data/kline_data";
    let filename = format!("{}/{}_{}_{}.json", folder, symbol.to_lowercase(), interval, month_str);
    // println!("Saving kline data to: {}", filename);
    if let Ok(json_str) = serde_json::to_string(&kline) {
        if let Err(e) = std::fs::create_dir_all(folder) {
            eprintln!("❌ 创建目录失败: {folder} - {e}");
            return;
        }

        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&filename)
        {
            let _ = writeln!(file, "{}", json_str);
        }
    }
}



pub fn load_kline_for_symbol_since(
    symbol: &str,
    interval: &str,
    since: DateTime<Utc>,
) -> Vec<KlineEvent> {
    let now = Utc::now();
    let mut results = Vec::new();

    let mut year = since.year();
    let mut month = since.month();

    let now_year = now.year();
    let now_month = now.month();

    while year < now_year || (year == now_year && month <= now_month) {
        let month_str = format!("{:04}{:02}", year, month);
        let path = format!(
            "data/kline_data/{}_{}_{}.json",
            symbol.to_lowercase(),
            interval,
            month_str
        );

        if let Ok(file) = File::open(Path::new(&path)) {
            let reader = BufReader::new(file);
            for line in reader.lines().flatten() {
                if let Ok(k) = serde_json::from_str::<KlineEvent>(&line) {
                    let ts = k.kline.start_time as i64;
                    let ts_dt = Utc.timestamp_millis(ts);
                    if ts_dt >= since {
                        results.push(k);
                    }
                }
            }
        }

        // 递增月份
        if month == 12 {
            year += 1;
            month = 1;
        } else {
            month += 1;
        }
    }

    results
}
