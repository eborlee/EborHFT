import sys
import os
import re
import json
import time
import requests
import pandas as pd
import mplfinance as mpf
from datetime import datetime, timedelta

# === 参数校验 ===
if len(sys.argv) != 3:
    print("Usage: python plot_img.py <trades.json> <output.png>")
    sys.exit(1)

trade_path = sys.argv[1]
# duration_str = sys.argv[2]
output_path = sys.argv[2]


# === 工具函数 ===
def extract_symbol_and_timeframe(path):
    match = re.search(r'([^/]+)_([0-9a-zA-Z]+)', os.path.dirname(path))
    if match:
        return match.group(1).upper(), match.group(2)
    raise ValueError("路径格式错误")

def timeframe_to_minutes(tf):
    unit = tf[-1]
    val = int(tf[:-1])
    if unit == 'm': return val
    elif unit == 'h': return val * 60
    elif unit == 'd': return val * 60 * 24
    raise ValueError(f"Unsupported tf: {tf}")

def get_latest_aligned_time(tf_minutes):
    now = datetime.utcnow().replace(second=0, microsecond=0)
    aligned_minute = (now.minute // tf_minutes) * tf_minutes
    return now.replace(minute=aligned_minute) - timedelta(minutes=tf_minutes)

def get_expected_start_time(timeframe):
    now = datetime.utcnow().replace(second=0, microsecond=0)
    delta_minutes = timeframe_to_minutes(timeframe)
    start_time = now - timedelta(minutes=delta_minutes)
    aligned_minute = (start_time.minute // delta_minutes) * delta_minutes
    return start_time.replace(minute=aligned_minute)


def find_missing_periods(klines_df, expected_start_time, tf_minutes):
    print(tf_minutes)
    # print(f"预期开始时间: {expected_start_time}")
    # print(f"预期结束时间: {get_latest_aligned_time(15)}")
    klines_df['timestamp'] = pd.to_datetime(klines_df['kline'].apply(lambda k: k['start_time']), unit='ms')
    klines_df = klines_df.sort_values(by='timestamp').set_index('timestamp')
    expected = pd.date_range(start=expected_start_time, end=get_latest_aligned_time(15), freq=f"{15}min")
    print(f"预期时间段: {len(expected)}")
    return expected.difference(klines_df.index)

def fetch_binance_continuous_klines(pair, interval, start_time, end_time):
    endpoint = 'https://fapi.binance.com/fapi/v1/continuousKlines'
    contract_type = 'PERPETUAL'
    limit = 1500
    result = []

    start_ms = int(start_time.timestamp() * 1000)
    end_ms = int(end_time.timestamp() * 1000)

    while start_ms < end_ms:
        params = {
            'pair': pair.upper(),
            'contractType': contract_type,
            'interval': interval,
            'startTime': start_ms,
            'endTime': min(end_ms, start_ms + limit * 15 * 60 * 1000),
            'limit': limit
        }
        response = requests.get(endpoint, params=params)
        if response.status_code != 200:
            raise Exception(f"Binance API error: {response.text}")
        data = response.json()
        if not data:
            break
        result.extend(data)
        start_ms = data[-1][0] + 15 * 60 * 1000  # 跳过最后一个 candle 开始时间
        time.sleep(0.3)  # 避免触发限速

    return result[:-1]


def group_klines_by_month(klines):
    grouped = {}
    for k in klines:
        ts = pd.to_datetime(k[0], unit='ms')
        key = f"{ts.year}{ts.month:02d}"
        grouped.setdefault(key, []).append(k)
    return grouped

def read_json_lines_safe(path):
    lines = []
    with open(path, 'r') as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            try:
                obj = json.loads(line)
                lines.append(obj)
            except json.JSONDecodeError:
                continue
    return lines

def write_grouped_klines(grouped, symbol):
    for ym, klist in grouped.items():
        path = f"data/kline_data/{symbol.lower()}_15m_{ym}.json"
        if os.path.exists(path):
            existing = read_json_lines_safe(path)  # 🔁 使用逐行加载
        else:
            existing = []
        # for k in existing:
        #     try:
        #         print(k['kline']['start_time'])
        #     except:
        #         print(k)
        existing_ts = set(k['kline']['start_time'] for k in existing)
        new_data = [k for k in klist if k[0] not in existing_ts]

        # 以 JSONL 格式逐行追加 new_data
        with open(path, 'a') as f:
            for k in new_data:
                obj = {
                    "event": "",
                    "event_time": "",
                    "pair": symbol,
                    "contract_type": "PERPETUAL",
                    "kline": {
                        "start_time": k[0],
                        "end_time": k[6],
                        "interval": "15m",
                        "first_trade_id": None,
                        "last_trade_id": None,
                        "open": k[1],
                        "close": k[4],
                        "high": k[2],
                        "low": k[3],
                        "volume": k[5],
                        "trade_count": k[8],
                        "is_final": True,
                        "quote_asset_volume": k[7],
                        "taker_buy_base_volume": k[9],
                        "taker_buy_quote_volume": k[10],
                        "ignore": k[11]
                    }
                }
                f.write(json.dumps(obj) + '\n')

def list_months_between(start: datetime, end: datetime):
    result = []
    y, m = start.year, start.month
    ey, em = end.year, end.month
    while (y < ey) or (y == ey and m <= em):
        result.append(f"{y}{m:02d}")
        if m == 12:
            y += 1
            m = 1
        else:
            m += 1
    return result



# symbol 由 trades.json 文件路径推断
symbol, tf = extract_symbol_and_timeframe(trade_path)  # 如 btcusdt_15m

duration_minutes = timeframe_to_minutes(tf)
expected_start = get_expected_start_time(tf)
expected_end = get_latest_aligned_time(15)
print(f"预期开始时间: {expected_start}")
print(f"预期结束时间: {expected_end}")

# 构造要读取的月份键
months = list_months_between(expected_start, expected_end)
print(months)
klines_all = []
for ym in months:  # months 是 ["202505", "202506", ...]
    path = f"data/kline_data/{symbol.lower()}_15m_{ym}.json"
    if os.path.exists(path):
        klines_all.extend(read_json_lines_safe(path))  # 按行读取，每行为一个 dict
    else:
        print(f"[warn] 未找到: {path}")

# 转 DataFrame（过滤掉非 dict / 异常行）
klines = pd.DataFrame([k for k in klines_all if isinstance(k, dict)])
print(f"读取到 {len(klines)} 条 K 线数据")


missing = find_missing_periods(klines, expected_start, duration_minutes)
print(f"缺失数据条数: {len(missing)}")
print(missing)
if not missing.empty:
    print(f"缺失时间段：{missing[0]} → {missing[-1]}")
    raw_klines = fetch_binance_continuous_klines(symbol, "15m", missing[0], missing[-1] + timedelta(minutes=15))
    grouped = group_klines_by_month(raw_klines)
    write_grouped_klines(grouped, symbol)
    print(f"已补全 {len(missing)} 条记录")

# === 重新读取数据 ===
klines_all = []
for ym in months:  # months 是 ["202505", "202506", ...]
    path = f"data/kline_data/{symbol.lower()}_15m_{ym}.json"
    if os.path.exists(path):
        klines_all.extend(read_json_lines_safe(path))  # 按行读取，每行为一个 dict
    else:
        print(f"[warn] 未找到: {path}")

# 转 DataFrame（过滤掉非 dict / 异常行）
klines = pd.DataFrame([k for k in klines_all if isinstance(k, dict)])
print(f"重新读取到 {len(klines)} 条 K 线数据")

# count = 0
# for k in klines.iterrows():
#     count += 1
#     if count > 1:
#         break
#     print(k)
    


# print(klines)
# === 处理 K线数据 ===
df = pd.DataFrame([{
    'timestamp': pd.to_datetime(k['kline']['start_time'], unit='ms'),
    'Open': float(k['kline']['open']),
    'High': float(k['kline']['high']),
    'Low': float(k['kline']['low']),
    'Close': float(k['kline']['close']),
    'Volume': float(k['kline']['volume']),
} for row, k in klines.iterrows()])
df.set_index('timestamp', inplace=True)

# === 处理成交数据 ===
trades = pd.read_json(trade_path)
trades['timestamp'] = pd.to_datetime(trades['event_time'], unit='ms')
trades = trades.sort_values(by='timestamp')
trades['quantity'] = trades['quantity'].astype(float)
trades['signed_qty'] = trades.apply(lambda row: -row['quantity'] if row['is_buyer_maker'] else row['quantity'], axis=1)

# === 累加方向性成交量 ===
position_series = (
    trades.set_index("timestamp")["signed_qty"]
    .resample("15min")
    .sum()
    .cumsum()
    .ffill()
)

concat = pd.concat([df, position_series.rename("cumsum_signed_qty")], axis=1)
concat['cumsum_signed_qty'] = concat['cumsum_signed_qty'].fillna(method='ffill')
concat = concat.dropna()
print(concat.shape)

# === 自定义图层：叠加 cumulative sum 折线 ===
apds = [mpf.make_addplot(concat['cumsum_signed_qty'], panel=0, color='blue', secondary_y=True)]

mpf.plot(
    concat[['Open', 'High', 'Low', 'Close', 'Volume']],
    type='candle',
    volume=True,
    style='classic',
    addplot=apds,
    figscale=1.2,
    figratio=(10, 6),
    title=f'[{symbol} - PERP - {tf}] Monitored Cumsum Position',
    savefig=output_path
)
