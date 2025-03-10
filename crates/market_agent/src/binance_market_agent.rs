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
// #[derive(Clone)]
pub struct BinanceMarketAgent {
    pub ws: Arc<Mutex<BinanceWebSocketClient>>,
    pub event_producer: QueueEventDispatcherProducer,
}

#[async_trait]
impl MarketAgent for BinanceMarketAgent {
    async fn start(self :Arc<Self>) -> Result<(), Box<dyn Error + Send>> {
        let (tx, mut rx) = tokio::sync::mpsc::channel::<(String, u128)>(1024);


        // 注册消息回调，将收到的文本消息判断是否包含 "depth"，若是则调用 on_depth 回调
        // 2. 启动工作任务：持续从通道中读取消息并处理
        let agent_clone = Arc::clone(&self);
        tokio::spawn(async move {
            while let Some((msg, received_timestamp)) = rx.recv().await {
                // 使用零拷贝方式解析 JSON
                let result: Result<BinanceEvent, _> = serde_json::from_slice(msg.as_bytes());
                match result {
                    Ok(event) => {
                        match event {
                            BinanceEvent::AggTrade(mut data) => {
                                data.received_timestamp = received_timestamp;
                                // 直接在工作任务中调用处理函数，无需额外 spawn
                                agent_clone.on_trade(data).await;
                            }
                            // 如果需要处理其他事件类型，可以添加分支
                        }
                    }
                    Err(e) => {
                        eprintln!("❌ JSON解析失败: {} - 原始消息: {}", e, msg);
                    }
                }
            }
        });

        // 3. 注册 WebSocket 消息回调，将消息发送到 mpsc 通道中
        {
            let tx_clone = tx.clone();
            let mut ws = self.ws.lock().await;
            ws.set_message_callback(move |msg: String| {
                let received_timestamp = get_timestamp_us();
                let tx_inner = tx_clone.clone();
                // 尽量使用 try_send 进行非阻塞发送，如果队列满了可考虑记录日志或其他处理方式
                if let Err(e) = tx_inner.try_send((msg, received_timestamp)) {
                    eprintln!("发送消息到工作队列失败: {:?}", e);
                }
            });
        }

        // 启动 WebSocket 的监听循环（内部包含断线重连逻辑）
        self.ws.lock().await.listen_loop().await?;
        Ok(())
    }


    async fn on_depth(&self, event: event::AggTradeEvent) { 
        // println!("MarketAgent on_depth 收到数据: {}", raw_data);
        // let mut producer = self.event_producer.lock().await;
        // // 将深度数据作为事件入队
        // producer.fire(EventType::Depth, EventPayload::AggTrade(event));
        self.event_producer.fire(EventType::Depth, EventPayload::AggTrade(event));
    }

    async fn on_trade(&self, event: event::AggTradeEvent) {
        // binance 的 aggTrade 数据
        // let mut producer = self.event_producer.lock().await;
        // // 将深度数据作为事件入队
        // producer.fire(EventType::AggTrade, EventPayload::AggTrade(event));
        self.event_producer.fire(EventType::AggTrade, EventPayload::AggTrade(event));
    }
}

impl BinanceMarketAgent {
    pub fn new(
        ws: BinanceWebSocketClient,
        event_producer: QueueEventDispatcherProducer,
    ) -> Self {
        Self {
            ws: Arc::new(Mutex::new(ws)),
            // event_producer: Arc::new(Mutex::new(event_producer)),
            event_producer: event_producer,
        }
    }
}