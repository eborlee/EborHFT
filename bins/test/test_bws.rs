use std::error::Error;
use tokio::time::sleep;
use std::time::Duration;
use feeder::websocket::WebSocket;
use feeder::websocket::BinanceWebSocketClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send>> {
    // 测试连接单一流，示例订阅 bnbusdt@aggTrade
    let streams = vec!["bnbusdt@aggTrade"];
    
    let mut client = BinanceWebSocketClient::new();
    
    // 建立连接
    client.connect(streams.clone()).await?;
    
    // 订阅消息（这里示例订阅单个流，实际订阅数量取决于 streams 数组）

    client.subscribe(vec!["btcusdt@aggTrade"]).await?;
    
    // 开始监听（注意：此处会持续打印接收到的消息）
    // 如果需要测试断线重连，可关闭连接或等待超过24小时后自动断连
    client.listen_loop().await?;
    
    Ok(())
}
