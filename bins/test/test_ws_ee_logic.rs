use ringbuf::{RingBuffer, Producer, Consumer};
use std::thread;
use std::time::Duration;

struct Event {
    data: String,
}

struct EventEngine {
    consumer: ringbuf::Consumer<Event>,
}

impl EventEngine {
    // 持续轮询消费环形缓冲区中的事件
    fn process(&mut self) {
        loop {
            // 如果有事件，则处理
            if let Some(event) = self.consumer.pop() {
                println!("处理事件: {}", event.data);
                // 可在这里添加更多业务逻辑
            } else {
                // 没有事件时稍微休眠，防止 CPU 占用过高
                thread::sleep(Duration::from_millis(10));
            }
        }
    }
}

struct BinanceWebSocketClient {
    producer: ringbuf::Producer<Event>,
}

impl BinanceWebSocketClient {
    // 模拟阻塞读取消息
    fn read_message(&self) -> String {
        thread::sleep(Duration::from_secs(1));
        "BinanceWebSocketClient收到数据".to_string()
    }

    // 持续监听 WS 数据，并将解析后的数据 push 到环形缓冲区中
    fn listen_loop(&mut self) {
        loop {
            let data = self.read_message();
            let event = Event { data };
            // 尝试将事件插入缓冲区，满了就丢弃或做其他处理
            if self.producer.push(event).is_err() {
                eprintln!("环形缓冲区已满，丢弃事件");
            }
        }
    }
}

fn main() {
    // 创建一个容量为 128 的环形缓冲区
    // let rb = HeapRb::new(128);
    let rb = RingBuffer::<Event>::new(128);
    // 拆分为生产者和消费者
    let (producer, consumer) = rb.split();

    // 分别构造 WS 客户端和事件引擎
    let mut ws_client = BinanceWebSocketClient { producer };
    let mut event_engine = EventEngine { consumer };

    // 在另一个线程中运行 WS 监听循环
    let ws_handle = thread::spawn(move || {
        ws_client.listen_loop();
    });

    // 主线程中执行事件处理循环
    event_engine.process();

    ws_handle.join().unwrap();
}
