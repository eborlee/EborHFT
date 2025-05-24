use crate::market_agent::MarketAgent;
use std::any::type_name;
use async_trait::async_trait;
use feeder::websocket::BinanceWebSocketClient;
use feeder::websocket::WebSocket;
use serde_json::Error as SerdeError;
use event_engine::event;
use event_engine::event::BinanceEvent;
use event_engine::event::EventType;
use event_engine::event::EventPayload;
use event_engine::event_dispatcher::QueueEventDispatcherProducer;
use std::sync::Arc;
// use tokio::sync::Mutex;
use std::sync::Mutex;
use std::error::Error;
use ringbuf::{RingBuffer, Producer, Consumer};
use tokio::runtime::Runtime;

use std::cell::RefCell;
use std::rc::Rc;

// use std::sync::atomic::{AtomicBool, Ordering};
// use std::sync::OnceLock;

use std::time::Duration;
use chrono::Local;



fn get_timestamp() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

fn get_timestamp_us() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_micros() // ✅ 以微秒（µs）为单位
}



/// BinanceMarketAgent 实现 MarketAgent 接口，封装 BinanceWebSocketClient 与事件分发器
// #[derive(Clone)]
pub struct BinanceMarketAgent {
    pub ws:  BinanceWebSocketClient,
    pub event_producer: QueueEventDispatcherProducer,
}


#[async_trait(?Send)]
impl MarketAgent for BinanceMarketAgent {
    async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // 注意这里依然在单线程场景下使用裸指针
        // 获取一个裸指针，用于在回调中操作 self
        let self_ptr = self as *mut BinanceMarketAgent;
        

        
        // 初始化 AtomicBool（只初始化一次）
        // FIRST_REAL_EVENT_PRINTED.get_or_init(|| AtomicBool::new(false));
        // 注册 WebSocket 消息回调
        self.ws.set_message_callback(move |msg: String| {
            let received_timestamp = get_timestamp_us();
            // 安全地通过裸指针获取可变引用
            let this = unsafe { &mut *self_ptr };
            match serde_json::from_slice(msg.as_bytes()) {
                Ok(BinanceEvent::AggTrade(mut data)) => {
                    data.received_timestamp = received_timestamp;
                    // println!("收到交易数据: {:?}", data);
                    this.on_trade(data);
                }
                Ok(BinanceEvent::Depth(mut data)) => {
                    // println!("收到深度数据: {:?}", data);
                    data.received_timestamp = received_timestamp;
                    this.on_depth(data);
                }
                Ok(BinanceEvent::Kline(mut data)) => {
                    data.received_timestamp = received_timestamp;
                    this.on_kline(data); // ✅ 你要定义这个方法
                }
                Err(e) => {
                    eprintln!("JSON解析失败: {} - 原始消息: {}", e, msg);
                }
                _ => {
                    eprintln!("收到未处理的事件类型: {}", msg);
                }
            }
        });

        // 直接启动 WebSocket 的监听循环
        self.ws.listen_loop().await?;
        Ok(())
    }
    

    fn on_trade(&mut self, event: event::AggTradeEvent) {
        self.event_producer.fire(EventType::AggTrade, EventPayload::AggTrade(event));
    }
    
    fn on_depth(&mut self, event: event::DepthEvent) {
        self.event_producer.fire(EventType::Depth, EventPayload::Depth(event));
    }

    fn on_kline(&mut self, event: event::KlineEvent) {
        self.event_producer.fire(EventType::Kline, EventPayload::Kline(event));
    }

    async fn subscribe(&mut self, streams: Vec<&str>) -> Result<(), Box<dyn Error>> {
        self.ws.subscribe(streams).await?;
        Ok(())
    }
}

struct AgentPtr(*mut BinanceMarketAgent);

// 告诉编译器：我确信此指针单线程使用、安全可跨线程搬移
unsafe impl Send for AgentPtr {}
unsafe impl Send for BinanceMarketAgent {}

impl BinanceMarketAgent {
    pub fn new(
        ws: BinanceWebSocketClient,
        event_producer: QueueEventDispatcherProducer,
    ) -> Self {
        Self {
            ws: ws,
            event_producer: event_producer,
        }
    }
}
