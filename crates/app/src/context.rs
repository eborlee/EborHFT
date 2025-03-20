// use std::sync::{Arc, Mutex};
use tokio::task;
use std::error::Error;

use event_engine::event_dispatcher::EventDispatcher;
use event_engine::event_dispatcher::AsyncQueueEventDispatcher;
use event_engine::event_dispatcher::QueueEventDispatcherProducer;
use event_engine::event_dispatcher::QueueEventDispatcherConsumer;
use event_engine::event::{EventType};
use event_engine::event_dispatcher::EventData;
use market_agent::market_agent::MarketAgent;
use market_agent::binance_market_agent::BinanceMarketAgent;
use feeder::websocket::WebSocket;
use feeder::websocket::BinanceWebSocketClient;
use tokio::runtime::Runtime;
use std::thread;

use crate::components::create_exchange_components;
use common::exchange::Exchange;

pub struct Context {
    // pub dispatcher: &'a AsyncQueueEventDispatcher,
    pub market_agent: Option<Box<dyn MarketAgent + Send>>,
    // pub ws_client: Box<dyn WebSocket>,
    // pub producer: &'a QueueEventDispatcherProducer,
    pub consumer: Option<QueueEventDispatcherConsumer>,
}

impl Context {
    /// 初始化 AppContext，只构造事件调度器和市场代理，不包含订单簿
    pub async fn new(exchange:Exchange, dispatcher_capacity: usize) -> Result<Self, Box<dyn Error>> {
        // 创建 dispatcher
        let dispatcher = AsyncQueueEventDispatcher::new(dispatcher_capacity);
        let (producer, mut consumer) = dispatcher.split();

        let exchange_components = create_exchange_components(exchange, producer).await?;
        // let ws_client = exchange_components.ws_client;
        let market_agent = exchange_components.market_agent;

        // ws_client.subscribe(vec!["btcusdt@depth@100ms"]).await?;

        

        Ok(Self {
            // dispatcher: &dispatcher,,
            market_agent: Some(market_agent),
            // ws_client,
            // producer: &producer,
            consumer: Some(consumer),
        })
    }

    /// 启动市场代理（异步运行）
    pub fn start_market_agent(&mut self) {
        let mut market_agent = self.market_agent.take().expect("market_agent is already taken");

        thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                market_agent.start().await;
            });
        });
        // Ok(())
    }

    /// 在独立线程中启动事件消费循环
    pub fn start_event_loop(&mut self) {
        let mut consumer = self.consumer.take().expect("consumer is already taken");
        thread::spawn(move || loop {
            consumer.process();
            // 根据需要可以添加 sleep 或 yield 以降低 CPU 占用
        });
    }

    /// 提供注册事件回调的接口，外部应用模块（如订单簿）可以通过此 API 注册回调
    pub fn register_callback<F>(&mut self, event_type: EventType, callback: Box<F>)
    where
        F: Fn(&EventData) + Send + Sync + 'static,
    {
        if let Some(ref mut consumer) = self.consumer {
            consumer.register(event_type, callback);
        } else {
            // 处理 None 的情况
        }
        
    }
}