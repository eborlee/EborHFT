use crate::models::{OrderBook, OrderSide, DepthSnapshot, parse_order_entry};
use std::error::Error;
use reqwest::Client;
use event_engine::event::EventType;
use event_engine::event::EventPayload;
use event_engine::event::DepthEvent;
use event_engine::event_dispatcher::EventData;


/// 订单簿维护引擎
pub struct OrderBookEngine {
    pub order_book: OrderBook,
    pub last_update_id: u64,
    // 用于在初始化前缓存增量事件
    pub update_buffer: Vec<DepthEvent>,
    // 回调，当订单簿更新时调用
    pub update_callbacks: Vec<Box<dyn Fn(&OrderBook) + Send + Sync>>,
    // 交易对，例如 "BTCUSDT"
    pub symbol: String,
    // 新增 flag，标识是否已经应用了第一个连续的深度更新事件
    pub continuous_started: bool,

}

impl OrderBookEngine {
    pub fn new(symbol: &str) -> Self {
        Self {
            order_book: OrderBook::new(),
            last_update_id: 0,
            update_buffer: Vec::new(),
            update_callbacks: Vec::new(),
            symbol: symbol.to_string(),
            continuous_started: false,

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
        let url = format!("https://fapi.binance.com/fapi/v1/depth?symbol={}&limit=1000", self.symbol);
        let client = Client::new();
        let response = client.get(&url).send().await?.json::<DepthSnapshot>().await?;
        Ok(response)
    }

    /// 初始化订单簿：调用 REST 获取快照，然后应用缓存中增量事件
    pub async fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let snapshot = self.fetch_depth_snapshot().await?;
        println!("修改前的last_update_id: {}", self.last_update_id);
        // println!("last update id的类型：{}", std::any::type_name_of_val(&self.last_update_id));
        self.last_update_id = snapshot.last_update_id;
        self.order_book = snapshot.to_order_book();
        println!("此时的last_update_id: {}", self.last_update_id);
        println!("此时buffer长度：{}", self.update_buffer.len());
        // 丢弃所有 final_update_id < last_update_id 的事件
        self.update_buffer.retain(|u| u.last_update_id >= self.last_update_id);
        println!("过滤后buffer长度：{}", self.update_buffer.len());
        // 找到第一个满足 U <= last_update_id <= u 的事件开始应用
        // 在遍历前克隆 update_buffer
        let updates = self.update_buffer.clone();
        for update in updates.iter() {
            println!("update.first_update_id: {}, self.last_update_id: {}, update.last_update_id: {}", update.first_update_id, self.last_update_id, update.last_update_id);
            if update.first_update_id <= self.last_update_id 
                && self.last_update_id <= update.last_update_id
            {
                println!("找到第一个满足 U <= last_update_id <= u 的事件开始应用");
                self.apply_update(update)?;
            }
        }
        self.update_buffer.clear();

        self.notify_update();
        Ok(())
    }

    /// 将增量更新缓存起来（如果尚未初始化）或直接应用（如果已经初始化）
    pub fn push_update(&mut self, event: EventData) -> Result<(), Box<dyn Error>> {
        // 仅处理深度事件
        if event.event_type != EventType::Depth {
            return Ok(());
        }
        // 从 event.data 中提取 DepthEvent
        let depth_event = match event.data {
            EventPayload::Depth(de) => {
                de
            },
            other => return Err("Expected Depth event payload".into()),
        };


        if self.last_update_id == 0 {
            // 未初始化时，缓存深度事件
            // println!("尚未初始化，缓存深度事件");
            self.update_buffer.push(depth_event);
            Ok(())
        } else {
            // 已初始化时，直接应用更新
            // println!("已初始化，直接应用更新");
            self.apply_update(&depth_event)
        }
    }

    /// 应用单个深度事件更新订单簿（这里将 DepthEvent 用作参数）
    fn apply_update(&mut self, update: &DepthEvent) -> Result<(), Box<dyn Error>> {
        if !self.continuous_started {
            // 还没有找到连续更新的起点，检查是否满足条件
            if update.first_update_id <= self.last_update_id && self.last_update_id <= update.last_update_id {
                println!("找到第一个满足连续条件的深度更新，作为连续更新起点");
                // 应用更新，不检查 previous_update_id
                for b in &update.bids {
                    let (price, qty) = parse_order_entry(b);
                    self.order_book.update_side(OrderSide::Buy, price, qty);
                }
                for a in &update.asks {
                    let (price, qty) = parse_order_entry(a);
                    self.order_book.update_side(OrderSide::Sell, price, qty);
                }
                self.last_update_id = update.last_update_id;
                self.order_book.event_time = Some(update.event_time);
                self.continuous_started = true;
                self.notify_update();
                Ok(())
            } else {
                // 还没达到连续更新的条件，忽略该更新
                println!("尚未找到连续更新起点，忽略当前更新");
                Ok(())
            }
        } else {
            // 已经找到连续更新的起点，正常检查连续性
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
            self.last_update_id = update.last_update_id;
            self.order_book.event_time = Some(update.event_time);
            self.notify_update();
            Ok(())
        }
    }
    

}
