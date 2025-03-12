use std::error::Error;
use tokio;
use orderbook::engine::OrderBookEngine;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // 创建订单簿维护引擎实例
    let mut engine = OrderBookEngine::new("BTCUSDT");

    // 注册回调，每次订单簿更新时打印最佳买卖价
    engine.register_callback(|order_book| {
        println!(
            "回调：最佳买价: {:?}, 最佳卖价: {:?}",
            order_book.top_n_bids(20),
            order_book.top_n_asks(20)
        );
    });

    // 通过 REST 接口获取深度快照（币安的真实数据）
    let snapshot = engine.fetch_depth_snapshot().await?;
    
    // 用 REST 快照初始化本地订单簿副本
    engine.last_update_id = snapshot.last_update_id;
    engine.order_book = snapshot.to_order_book();
    
    println!("初始化订单簿成功，当前状态：");
    println!("最佳20买价: {:?}, 最佳20卖价: {:?}", engine.order_book.top_n_bids(20), engine.order_book.top_n_asks(20));
    println!("完整买价: {:?}", engine.order_book.bids());

    // 这里后续可以添加 websocket 更新模拟或其他逻辑测试
    Ok(())
}
