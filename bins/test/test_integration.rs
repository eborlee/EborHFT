use feeder::websocket::BinanceWebSocketClient;
use tokio;

use event_engine::event::EventType;
use event_engine::event::EventPayload;

use event_engine::event_dispatcher::{EventDispatcher};
use event_engine::event_dispatcher_spsc::AsyncQueueEventDispatcherSPSC;
use event_engine::event_dispatcher_mpsc::AsyncQueueEventDispatcherMPSC;


use market_agent::market_agent::MarketAgentSPSC;
use market_agent::binance_market_agent::BinanceMarketAgentSPSC;
use market_agent::market_agent::MarketAgentMPSC;
use market_agent::binance_market_agent::BinanceMarketAgentMPSC;
use market_agent::{create_market_agent_with_consumer, AgentType, Exchange, MarketAgentEnum, MarketAgentConsumerEnum};


use feeder::websocket::WebSocket;
use feeder::websocket::BinanceWebSocketClient;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use chrono::Local;
use tokio::runtime::Builder;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建 WebSocket 客户端（此处仅支持 Binance）
    let ws = BinanceWebSocketClient::new();

    // 选择 agent 类型（Spsc 或 Mpsc）、交易所（目前仅 Binance）和队列容量
    let agent_type = AgentType::Mpsc; // 或 AgentType::Mpsc
    let exchange = Exchange::Binance;
    let capacity = 200;

    // 调用工厂函数，同时获取 agent 和 consumer
    let (mut market_agent, mut consumer) =
        create_market_agent_with_consumer(exchange, agent_type, ws, capacity);

    // 启动 agent（这里启动在一个 tokio 任务中）
    tokio::spawn(async move {
        if let Err(e) = market_agent.start().await {
            eprintln!("MarketAgent 错误：{}", e);
        }
    });

    // 在主线程中定时处理消费端队列
    loop {
        match &mut consumer {
            MarketAgentConsumerEnum::Spsc(consumer) => consumer.process(),
            MarketAgentConsumerEnum::Mpsc(consumer) => consumer.process(),
        }
        tokio::time::sleep(std::time::Duration::from_millis(1)).await;
    }
}