use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};
use ordered_float::OrderedFloat;

/// 订单方向：买或卖
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrderSide {
    Buy,
    Sell,
}

/// 表示订单簿一侧（买或卖）
/// - key: 价格
/// - value: 该价格的累计挂单量
#[derive(Debug, Clone)]
pub struct OrderBookSide {
    pub orders: BTreeMap<OrderedFloat<f64>, f64>,
}


impl OrderBookSide {
    pub fn new() -> Self {
        Self {
            orders: BTreeMap::new(),
        }
    }

    /// 更新或删除挂单
    /// 如果 quantity == 0，则删除该价位；否则覆盖为新的数量
    pub fn update(&mut self, price: f64, quantity: f64) {
        let key = OrderedFloat(price);
        if quantity == 0.0 {
            self.orders.remove(&OrderedFloat(price));
        } else {
            self.orders.insert(OrderedFloat(price), quantity);
        }
    }
}

/// 整体订单簿：包含买盘 (bids) 和卖盘 (asks)
#[derive(Debug, Clone)]
pub struct OrderBook {
    pub bids: OrderBookSide,
    pub asks: OrderBookSide,
    pub event_time: Option<u64>,
}

impl OrderBook {
    pub fn new() -> Self {
        Self {
            bids: OrderBookSide::new(),
            asks: OrderBookSide::new(),
            event_time: None,
        }
    }

    /// 根据 side 更新指定价位的数量
    pub fn update_side(&mut self, side: OrderSide, price: f64, quantity: f64) {
        match side {
            OrderSide::Buy => self.bids.update(price, quantity),
            OrderSide::Sell => self.asks.update(price, quantity),
        }
    }

    pub fn best_bid(&self) -> Option<(f64, f64)> {
        self.bids.orders.iter().rev().next().map(|(p, &q)| (p.0, q))
    }
    
    pub fn best_ask(&self) -> Option<(f64, f64)> {
        self.asks.orders.iter().next().map(|(p, &q)| (p.0, q))
    }
    
    pub fn top_n_bids(&self, n: usize) -> Vec<(f64, f64)> {
        self.bids.orders
            .iter()
            .rev() // 反向迭代，最高买价在前
            .take(n)
            .map(|(p, &q)| (p.0, q))
            .collect()
    }
    
    pub fn top_n_asks(&self, n: usize) -> Vec<(f64, f64)> {
        self.asks.orders
            .iter()
            .take(n)
            .map(|(p, &q)| (p.0, q))
            .collect()
    }

    /// 返回买单容器的只读引用
    pub fn bids(&self) -> Vec<(f64, f64)> {
        self.bids.orders
            .iter()
            .rev()
            .map(|(p, &q)| (p.0, q))
            .collect()
    }

    /// 返回卖单容器的只读引用
    pub fn asks(&self) -> Vec<(f64, f64)> {
        self.asks.orders
            .iter()
            .map(|(p, &q)| (p.0, q))
            .collect()
    }
    
}


/// 币安深度快照 (REST API 获取)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DepthSnapshot {
    #[serde(rename = "lastUpdateId")]
    pub last_update_id: u64,

    #[serde(rename = "E")]
    pub event_time: Option<u64>, // 可选字段

    #[serde(rename = "T")]
    pub match_time: Option<u64>, // 可选字段

    #[serde(rename = "bids")]
    pub bids: Vec<[String; 2]>, // Vec<[price, quantity]>

    #[serde(rename = "asks")]
    pub asks: Vec<[String; 2]>, // Vec<[price, quantity]>
}

/// 币安增量深度更新 (WebSocket 接收)
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DepthUpdateEvent {
    #[serde(rename = "U")]
    pub first_update_id: u64,         // U
    #[serde(rename = "u")]
    pub final_update_id: u64,         // u
    #[serde(rename = "pu")]
    pub previous_update_id: u64,      // pu
    #[serde(rename = "b")]
    pub bids: Vec<[String; 2]>,       // [price, quantity]
    #[serde(rename = "a")]
    pub asks: Vec<[String; 2]>,       // [price, quantity]
}

/// 辅助函数：将 [String; 2] 转换为 (f64, f64)
pub fn parse_order_entry(entry: &[String; 2]) -> (f64, f64) {
    let price = entry[0].parse::<f64>().unwrap_or(0.0);
    let qty = entry[1].parse::<f64>().unwrap_or(0.0);
    (price, qty)
}

/// 将 `DepthSnapshot` 转换为 `OrderBook`
impl DepthSnapshot {
    /// 解析字符串格式的 bids/asks 数据为 (f64, f64)
    pub fn to_order_book(self) -> OrderBook {
        let mut order_book = OrderBook::new();

        for bid in self.bids {
            let (price, quantity) = parse_order_entry(&bid);
            order_book.update_side(OrderSide::Buy, price, quantity);
        }

        for ask in self.asks {
            let (price, quantity) = parse_order_entry(&ask);
            order_book.update_side(OrderSide::Sell, price, quantity);
        }

        // 将快照中的事件时间赋值给订单簿
        order_book.event_time = self.event_time;

        order_book
    }
}