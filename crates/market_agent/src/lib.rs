pub mod market_agent;
pub mod binance_market_agent;
pub mod agent_factory;

pub use agent_factory::{create_market_agent_with_consumer, AgentType, Exchange, MarketAgentConsumerEnum, MarketAgentEnum};