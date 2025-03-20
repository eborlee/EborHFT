// runtime.rs
use std::error::Error;
use crate::context::Context;
use event_engine::event::EventType;
use common::exchange::Exchange;
use event_engine::event_dispatcher::EventData;
use tokio;

pub struct Runtime {
    pub context: Context,
    // 可以在此扩展其他公共配置、日志、状态等
}

impl Runtime {
    /// 创建 Runtime 实例，并初始化上下文
    pub async fn new(exchange:Exchange, dispatcher_capacity: usize) -> Result<Self, Box<dyn Error>> {
        let context = Context::new(exchange, dispatcher_capacity).await?;
        Ok(Self { context })
    }

    /// 启动服务：启动市场代理、事件循环等
    pub async fn start_service(&mut self) -> Result<(), Box<dyn Error>> {
        self.context.start_market_agent();
        self.context.start_event_loop();
        println!("服务已启动！按 Ctrl+C 退出。");
    
        // 主线程等待 Ctrl+C 信号，从而保持运行状态
        tokio::signal::ctrl_c().await?;
        println!("收到退出信号，程序结束。");
        Ok(())
    }

    /// 对外暴露注册事件回调的接口
    pub fn register_event_callback<F>(&mut self, event_type: EventType, callback: Box<F>)
    where
        F: Fn(&EventData) + Send + Sync + 'static,
    {
        self.context.register_callback(event_type, callback);
    }

    pub async fn subscribe(&mut self, streams: Vec<&str>) -> Result<(), Box<dyn Error>> {
        // 内部处理 Option，不用让使用者处理 unwrap
        if let Some(agent) = self.context.market_agent.as_mut() {
            agent.subscribe(streams).await
        } else {
            Err("market_agent is None".into())
        }
    }
}
