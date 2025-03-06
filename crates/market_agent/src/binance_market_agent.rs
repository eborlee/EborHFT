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
use tokio::sync::Mutex;
use std::error::Error;

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
#[derive(Clone)]
pub struct BinanceMarketAgent {
    pub ws: Arc<Mutex<BinanceWebSocketClient>>,
    pub event_producer: Arc<Mutex<QueueEventDispatcherProducer>>,
}

#[async_trait]
impl MarketAgent for BinanceMarketAgent {
    async fn start(&self) -> Result<(), Box<dyn Error + Send>> {
        // 注册消息回调，将收到的文本消息判断是否包含 "depth"，若是则调用 on_depth 回调
        let agent_clone = Arc::new(self.clone());
        {
            let ws_agent = Arc::clone(&agent_clone);
            let mut ws = self.ws.lock().await;
            ws.set_message_callback(move |msg: String| {
                let received_timestamp = get_timestamp_us();
                let agent = Arc::clone(&ws_agent);
                
                tokio::spawn(async move {
                    let event: Result<BinanceEvent, SerdeError> = serde_json::from_str(&msg);
                    match event {
                        Ok(event) => {
                            match event {
                                BinanceEvent::AggTrade(mut data) => {
                                    data.received_timestamp = received_timestamp; 
                                    agent.on_trade(data).await;
                                }
                                // BinanceEvent::DepthUpdate(data) => {
                                //     agent.on_depth(data).await;
                                // }
                            }
                        }
                        Err(e) => {
                            eprintln!("❌ JSON 解析失败: {} - 原始消息: {}", e, msg);
                        }
                    }
                });
            });
        }
        // 启动 ws 的监听循环（内部包含断线重连逻辑）
        self.ws.lock().await.listen_loop().await?;
        Ok(())
    }


    async fn on_depth(&self, event: event::AggTradeEvent) {
        // 此处可添加对 raw_data 的进一步解析、转换
        // println!("MarketAgent on_depth 收到数据: {}", raw_data);
        let mut producer = self.event_producer.lock().await;
        // 将深度数据作为事件入队
        producer.fire(EventType::Depth, EventPayload::AggTrade(event));
    }

    async fn on_trade(&self, event: event::AggTradeEvent) {
        // binance 的 aggTrade 数据
        let mut producer = self.event_producer.lock().await;
        // 将深度数据作为事件入队
        producer.fire(EventType::AggTrade, EventPayload::AggTrade(event));
    }
}

impl BinanceMarketAgent {
    pub fn new(
        ws: BinanceWebSocketClient,
        event_producer: QueueEventDispatcherProducer,
    ) -> Self {
        Self {
            ws: Arc::new(Mutex::new(ws)),
            event_producer: Arc::new(Mutex::new(event_producer)),
        }
    }
}