use std::sync::{Arc, Mutex};
use event_engine::event_dispatcher_spsc::QueueEventDispatcherSPSCConsumer;
use event_engine::event_dispatcher_mpsc::QueueEventDispatcherMPSCConsumer;
use event_engine::event_dispatcher_spsc::QueueEventDispatcherSPSCProducer;
use event_engine::event_dispatcher_mpsc::QueueEventDispatcherMPSCProducer;


use event_engine::event_dispatcher_spsc::AsyncQueueEventDispatcherSPSC;
use event_engine::event_dispatcher_mpsc::AsyncQueueEventDispatcherMPSC;

use crate::binance_market_agent::BinanceMarketAgentSPSC;
use crate::binance_market_agent::BinanceMarketAgentMPSC;

use feeder::websocket::BinanceWebSocketClient;

pub enum Exchange {
    Binance,
}


/// Agent 类型选择枚举
pub enum AgentType {
    Spsc,
    Mpsc,
}

pub enum MarketAgentEnum {
    Spsc(BinanceMarketAgentSPSC),
    Mpsc(Arc<BinanceMarketAgentMPSC>),
}

// 枚举封装两种消费端的类型
pub enum MarketAgentConsumerEnum {
    Spsc(QueueEventDispatcherSPSCConsumer),
    Mpsc(QueueEventDispatcherMPSCConsumer),
}


/// 工厂函数，根据用户参数创建不同的 agent
pub fn create_market_agent_with_consumer(
    exchange: Exchange,
    agent_type: AgentType,
    ws: BinanceWebSocketClient,
    capacity: usize,
) -> (MarketAgentEnum, MarketAgentConsumerEnum) {
    match exchange {
        Exchange::Binance => {
            match agent_type {
                AgentType::Spsc => {
                    let async_dispatcher =
                        AsyncQueueEventDispatcherSPSC::new(capacity);
                    let (producer, consumer) = async_dispatcher.split();
                    let agent = BinanceMarketAgentSPSC::new(ws, producer);
                    (
                        MarketAgentEnum::Spsc(agent),
                        MarketAgentConsumerEnum::Spsc(consumer),
                    )
                }
                AgentType::Mpsc => {
                    let async_dispatcher =
                        AsyncQueueEventDispatcherMPSC::new(capacity);
                    let (producer, consumer) = async_dispatcher.split();
                    let agent = BinanceMarketAgentMPSC::new(ws, producer);
                    (
                        MarketAgentEnum::Mpsc(Arc::new(agent)),
                        MarketAgentConsumerEnum::Mpsc(consumer),
                    )
                }
            }
        }
        // 如果未来扩展其他交易所，在此添加分支
        //_ => unimplemented!("尚未支持该交易所"),
    }
}

// 辅助函数：获取当前时间（µs）
fn get_timestamp_us() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_micros()
}