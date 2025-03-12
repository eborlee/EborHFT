use crate::models::{OrderBook, OrderSide, DepthSnapshot, DepthUpdateEvent, parse_order_entry};
use std::error::Error;
use reqwest::Client;


/// 订单簿维护引擎
pub struct OrderBookEngine {
    pub order_book: OrderBook,
    pub last_update_id: u64,
    // 用于在初始化前缓存增量事件
    pub update_buffer: Vec<DepthUpdateEvent>,
    // 回调，当订单簿更新时调用
    pub update_callbacks: Vec<Box<dyn Fn(&OrderBook) + Send + Sync>>,
    // 交易对，例如 "BTCUSDT"
    pub symbol: String,

}

impl OrderBookEngine {
    pub fn new(symbol: &str) -> Self {
        Self {
            order_book: OrderBook::new(),
            last_update_id: 0,
            update_buffer: Vec::new(),
            update_callbacks: Vec::new(),
            symbol: symbol.to_string(),
        }
    }

    /// 注册一个新的回调
    pub fn register_callback<F>(&mut self, callback: F)
    where
        F: Fn(&OrderBook) + Send + Sync + 'static,
    {
        self.update_callbacks.push(Box::new(callback));
    }

    /// 通知所有回调
    fn notify_update(&self) {
        for callback in &self.update_callbacks {
            callback(&self.order_book);
        }
    }

    /// 通过 REST API 获取深度快照
    pub async fn fetch_depth_snapshot(&self) -> Result<DepthSnapshot, Box<dyn std::error::Error>> {
        let url = format!("https://fapi.binance.com/fapi/v1/depth?symbol={}&limit=2000", self.symbol);
        let client = Client::new();
        let response = client.get(&url).send().await?.json::<DepthSnapshot>().await?;
        Ok(response)
    }

    /// 初始化订单簿：调用 REST 获取快照，然后应用缓存中增量事件
    pub async fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let snapshot = self.fetch_depth_snapshot().await?;
        self.last_update_id = snapshot.last_update_id;
        self.order_book = snapshot.to_order_book();

        // 丢弃所有 final_update_id < last_update_id 的事件
        self.update_buffer.retain(|u| u.final_update_id >= self.last_update_id);

        // 找到第一个满足 U <= last_update_id <= u 的事件开始应用
        // 在遍历前克隆 update_buffer
        let updates = self.update_buffer.clone();
        for update in updates.iter() {
            if update.first_update_id <= self.last_update_id 
                && self.last_update_id <= update.final_update_id
            {
                self.apply_update(update)?;
            }
        }
        self.update_buffer.clear();

        self.notify_update();
        Ok(())
    }

    /// 将增量更新缓存起来（如果尚未初始化）或直接应用（如果已经初始化）
    pub fn push_update(&mut self, update: DepthUpdateEvent) -> Result<(), Box<dyn std::error::Error>> {
        if self.last_update_id == 0 {
            // 未初始化时先缓存事件
            self.update_buffer.push(update);
            Ok(())
        } else {
            // 已初始化，直接应用
            self.apply_update(&update)
        }
    }

    /// 应用单个增量更新事件到订单簿，并进行连续性验证
    fn apply_update(&mut self, update: &DepthUpdateEvent) -> Result<(), Box<dyn std::error::Error>> {
        if update.previous_update_id != self.last_update_id {
            return Err("更新连续性验证失败，需要重新初始化".into());
        }
        for b in &update.bids {
            let (price, qty) = parse_order_entry(b);
            self.order_book.update_side(OrderSide::Buy, price, qty);
        }
        for a in &update.asks {
            let (price, qty) = parse_order_entry(a);
            self.order_book.update_side(OrderSide::Sell, price, qty);
        }
        self.last_update_id = update.final_update_id;

        self.notify_update();
        Ok(())
    }

}
