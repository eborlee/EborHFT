use event_engine::event_dispatcher::QueueEventDispatcherProducer;
use market_agent::market_agent::MarketAgent;
use market_agent::binance_market_agent::BinanceMarketAgent;
use feeder::websocket::WebSocket;
use feeder::websocket::BinanceWebSocketClient;
use common::exchange::Exchange;

// 定义一个结构体存放两个模块的实例
pub struct ExchangeComponents {
    // pub ws_client: Box<dyn WebSocket>,
    pub market_agent: Box<dyn MarketAgent + Send>,
}

pub async fn create_exchange_components(
    exchange: Exchange,
    producer: QueueEventDispatcherProducer, 
) -> Result<ExchangeComponents, Box<dyn std::error::Error>> {
    match exchange {
        Exchange::Binance => {
            // 生成 Binance 的 websocket 客户端和 market agent
            let mut ws_client = BinanceWebSocketClient::new();
            // ws_client.connect(Vec::<&str>::new()).await?;
            ws_client.connect(vec!["btcusdt@depth@100ms"]).await?;
            // ws_client.subscribe(vec!["btcusdt@depth@100ms"]).await?;
            let market_agent = BinanceMarketAgent::new(ws_client, producer);
            Ok(ExchangeComponents {
                // ws_client: Box::new(ws_client),
                market_agent: Box::new(market_agent),
            })
        }
        
    }
}