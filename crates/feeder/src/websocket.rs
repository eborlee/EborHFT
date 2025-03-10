use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use std::error::Error;
use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tokio::time::sleep;
use tokio_tungstenite::{connect_async, tungstenite::Message, WebSocketStream};
use tokio_tungstenite::MaybeTlsStream;
use std::io;
use std::cell::RefCell;
use std::rc::Rc;
use url::Url;

#[async_trait(?Send)]
pub trait WebSocket {
    /// 建立连接：根据传入的订阅流构造 URL 并连接
    async fn connect(&mut self, streams: Vec<&str>) -> Result<(), Box<dyn Error>>;
    /// 发送文本消息
    async fn send(&mut self, msg: &str) -> Result<(), Box<dyn Error >>;
    /// 读取一条消息（包含控制帧）
    async fn read_message(&mut self) -> Result<Message, Box<dyn Error >>;
    /// 监听循环：处理消息、回复 ping、检测断线等
    async fn listen_loop(&mut self) -> Result<(), Box<dyn Error >>;
    /// 发送订阅消息（单个连接最多200个流，且受限于每秒10条消息）
    async fn subscribe(&mut self, streams: Vec<&str>) -> Result<(), Box<dyn Error >>;
}

// 修改 BinanceWebSocketClient，增加一个 on_message 回调属性
type MessageCallback = Box<dyn FnMut(String) + 'static>;

pub struct BinanceWebSocketClient {
    /// 内部保存连接后的 WebSocketStream
    ws_stream: Option<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    /// 记录连接建立时间，用于判断24小时有效期
    connection_start: Option<Instant>,
    on_message_callback: Option<MessageCallback>,
}

impl BinanceWebSocketClient {
    pub fn new() -> Self {
        Self {
            ws_stream: None,
            connection_start: None,
            on_message_callback: None,
        }
    }

    /// 根据订阅流列表构造连接 URL
    /// 单一流：wss://fstream.binance.com/ws/<streamName>
    /// 多流：wss://fstream.binance.com/stream?streams=/<stream1>/<stream2>/...
    fn build_url(&self, streams: &[&str]) -> String {
        let base = "wss://fstream.binance.com";
        if streams.len() == 1 {
            format!("{}/ws/{}", base, streams[0].to_lowercase())
        } else {
            let combined = streams
                .iter()
                .map(|s| s.to_lowercase())
                .collect::<Vec<String>>()
                .join("/");
            format!("{}/stream?streams=/{}", base, combined)
        }
    }

    /// 回复 ping 帧，发送 pong 帧（允许发送不成对的pong帧）
    async fn send_pong(&mut self, data: Vec<u8>) -> Result<(), Box<dyn Error >> {
        if let Some(ref mut ws) = self.ws_stream {
            ws.send(Message::Pong(data))
                .await
                .map_err(|e| -> Box<dyn std::error::Error > { Box::new(e) })?;

            Ok(())
        } else {
            Err(Box::new(io::Error::new(io::ErrorKind::Other, "WebSocket 未连接")))

        }
    }

    /// 设置消息回调
    pub fn set_message_callback<F>(&mut self, callback: F)
    where
        F: FnMut(String) + 'static,
    {
        self.on_message_callback = Some(Box::new(callback));
    }
}

#[async_trait(?Send)]
impl WebSocket for BinanceWebSocketClient {
    async fn connect(&mut self, streams: Vec<&str>) -> Result<(), Box<dyn Error >> {
        // 检查订阅流数量
        if streams.len() > 200 {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "单个连接最多订阅 200 个 Streams")))
        }
        let url_str = self.build_url(&streams);
        println!("尝试连接: {}", url_str);
        let url = Url::parse(&url_str)
                    .map_err(|e| Box::<dyn std::error::Error >::from(Box::new(e)))?;

        let (ws_stream, _) = connect_async(url).await
                    .map_err(|e| Box::<dyn std::error::Error >::from(Box::new(e)))?;
        self.ws_stream = Some(ws_stream);
        self.connection_start = Some(Instant::now());
        println!("连接成功");
        Ok(())
    }

    async fn send(&mut self, msg: &str) -> Result<(), Box<dyn Error >> {
        if let Some(ref mut ws) = self.ws_stream {
            ws.send(Message::Text(msg.to_string())).await
                    .map_err(|e| Box::<dyn std::error::Error >::from(Box::new(e)))?;
            Ok(())
        } else {
            Err(Box::new(io::Error::new(io::ErrorKind::Other, "WebSocket 未连接")))
        }
    }

    async fn read_message(&mut self) -> Result<Message, Box<dyn Error >> {
        if let Some(ref mut ws) = self.ws_stream {
            if let Some(msg) = ws.next().await {
                let m = msg.map_err(|e| Box::new(e) as Box<dyn std::error::Error >)?;
                Ok(m)
            } else {
                Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "WebSocket 已关闭")))
            }
        } else {
            Err(Box::new(io::Error::new(io::ErrorKind::Other, "WebSocket 未连接")))
        }
    }

    async fn listen_loop(&mut self) -> Result<(), Box<dyn Error >> {
        // 外层循环用于断线重连与定时重连（24小时断线重连）
        loop {
            // 判断连接是否超过24小时，有效期小于24小时
            if let Some(start) = self.connection_start {
                if start.elapsed() > Duration::from_secs(24 * 3600) {
                    println!("连接已超过24小时，准备重连...");
                    self.ws_stream = None;
                }
            }
            // 如无连接则退出内层监听循环，进入重连流程
            if self.ws_stream.is_none() {
                break;
            }
            // 内层循环读取消息
            // println!("开始监听消息");
            match self.read_message().await {
                Ok(message) => {
                    match message {
                        Message::Text(text) => {
                            // 如果是组合 streams，payload 格式为 {"stream": "...", "data": ...}
                            // println!("收到文本消息: {}", text);
                            // 此处可根据业务解析并分发到 on_depth / on_trade 等回调
                            if let Some(ref mut callback) = self.on_message_callback {
                                callback(text);
                            }
                        }
                        Message::Ping(data) => {
                            println!("收到 ping, 回复 pong");
                            self.send_pong(data)
                                .await
                                .map_err(|e| Box::<dyn std::error::Error>::from(e))?;


                        }
                        Message::Pong(_) => {
                            println!("收到 pong");
                        }
                        Message::Binary(bin) => {
                            println!("收到二进制消息: {:?}", bin);
                        }
                        Message::Close(frame) => {
                            println!("收到关闭消息: {:?}", frame);
                            self.ws_stream = None;
                            break;
                        }
                        _ => {
                            // println!("收到default消息: {:?}", message);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("读取消息错误: {}，准备重连", e);
                    self.ws_stream = None;
                    break;
                }
            }
        }
        // 断线重连前等待3秒，防止频繁重连
        sleep(Duration::from_secs(3)).await;
        Ok(())
    }

    async fn subscribe(&mut self, streams: Vec<&str>) -> Result<(), Box<dyn Error >> {
        // 检查订阅流数量
        if streams.len() > 200 {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "单个连接最多订阅 200 个 Streams")))
        }
        // Binance 订阅消息格式示例：
        // { "method": "SUBSCRIBE", "params": ["stream1", "stream2"], "id": 1 }
        // let streams_lower: Vec<String> = streams.iter().map(|s| s.collect();
        let subscribe_msg = json!({
            "method": "SUBSCRIBE",
            "params": streams,
            "id": 1
        });
        let msg_text = subscribe_msg.to_string();
        // 简单速率控制：每条消息间隔至少100ms，确保不超过每秒10条
        sleep(Duration::from_millis(100)).await;
        self.send(&msg_text).await?;
        println!("发送订阅消息: {}", msg_text);
        Ok(())
    }
}
