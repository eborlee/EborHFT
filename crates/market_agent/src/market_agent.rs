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
#[async_trait]
pub trait MarketAgent:{
    /// 启动市场代理，通常需要注册回调并启动 ws 的监听循环
    async fn start(self: Arc<Self>) -> Result<(), Box<dyn Error>>;

    /// 收到深度数据时的回调，将原始数据解析后入队事件
    async fn on_depth(&self, event: event::AggTradeEvent);

    // 收到成交数据时的回调，将原始数据解析后入队事件
    async fn on_trade(&self, event: event::AggTradeEvent);

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

#[async_trait(?Send)]
pub trait MarketAgentSPSC: MarketAgent {
    // 可以增加一些 SPSC 特有的方法
    // 比如：启动同步版本的代理（这里仅做示例，实际设计中可根据需要调整）
    async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>>;
}

#[async_trait()]
pub trait MarketAgentMPSC: MarketAgent {
    async fn start(Arc<Self>) -> Result<(), Box<dyn std::error::Error>>;
}



