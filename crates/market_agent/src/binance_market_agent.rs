use crate::market_agent::MarketAgent;

use async_trait::async_trait;
use feeder::websocket::BinanceWebSocketClient;
use feeder::websocket::WebSocket;

use event_engine::event::EventType;
use event_engine::event_dispatcher::QueueEventDispatcherProducer;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::error::Error;

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
        let agent_clone = self.clone();
        {
            let mut ws = self.ws.lock().await;
            ws.set_message_callback(move |msg: String| {
                let agent = agent_clone.clone();
                tokio::spawn(async move {
                    // 根据实际业务，可进一步解析 msg，此处简单判断是否包含 "depth"
                    // if msg.contains("depth") {
                    //     agent.on_depth(msg).await;
                    // }
                    agent.on_depth(msg).await;
                });
            });
        }
        // 启动 ws 的监听循环（内部包含断线重连逻辑）
        self.ws.lock().await.listen_loop().await?;
        Ok(())
    }

    async fn on_depth(&self, raw_data: String) {
        // 此处可添加对 raw_data 的进一步解析、转换
        println!("MarketAgent on_depth 收到数据: {}", raw_data);
        let mut producer = self.event_producer.lock().await;
        // 将深度数据作为事件入队
        producer.fire(EventType::Depth, vec![raw_data]);
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