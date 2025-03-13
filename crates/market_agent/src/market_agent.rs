// market_agent.rs

use async_trait::async_trait;
use feeder::websocket::BinanceWebSocketClient;
use event_engine::event;
use event_engine::event::EventType;
use event_engine::event_dispatcher::QueueEventDispatcherProducer;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::error::Error;

/// MarketAgent 定义了市场代理所需实现的接口
#[async_trait(?Send)]
pub trait MarketAgent {
    // 启动市场代理 单生产者单消费者下，不需要声明为Send
    async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>>;

    fn on_depth(&mut self, event: event::DepthEvent);

    fn on_trade(&mut self, event: event::AggTradeEvent);

    // // 收到 K 线数据时的回调，将原始数据解析后入队事件
    // async fn on_kline(&self, raw_data: String);

    // // 收到 ticker 数据时的回调，将原始数据解析后入队事件
    // async fn on_ticker(&self, raw_data: String);

    // // 收到归集交易数据时的回调，将原始数据解析后入队事件
    // async fn on_agg_trade(&self, raw_data: String);

    // // 订阅指定的流
    // async fn subscribe(&self, streams: Vec<&str>) -> Result<(), Box<dyn Error + Send>>;

    // // 取消订阅指定的流
    // async fn unsubscribe(&self, streams: Vec<&str>) -> Result<(), Box<dyn Error + Send>>;
}
