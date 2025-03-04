// use market_agent::market_event_engine;
use event_engine::event;
use event_engine::event_dispatcher::QueueEventDispatcher;
use event_engine::event_dispatcher::AsyncQueueEventDispatcher;
use event_engine::event_dispatcher::QueueEventDispatcherConsumer;
use event_engine::event_dispatcher::QueueEventDispatcherProducer;
use event_engine::event_dispatcher::EventData;
use event_engine::event_dispatcher::EventDispatcher; // 如果需要使用 EventDispatcher trait
use std::thread;
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use chrono::Local;

use std::any::Any;
// fn main() {
//     // 创建一个容量为 5 的事件分发器
//     let mut dispatcher = QueueEventDispatcher::new(5);
    
//     // 为 EventA 注册一个回调
//     dispatcher.register(event::EventType::Trade, Box::new(|event: &EventData| {
//         println!("收到 成交数据更新 事件，数据：{:?}", event.data);
//     }));
    
//     // 为 EventB 注册一个回调
//     dispatcher.register(event::EventType::Depth, Box::new(|event: &EventData| {
//         println!("收到 深度数据更新 事件，数据：{:?}", event.data);
//     }));
//     // 触发一个没有参数的 EventA 事件
//     dispatcher.fire(event::EventType::Depth, vec!["Foo", "Bar"]);
    
//     // 处理事件队列，依次调用注册的回调函数
//     dispatcher.process();
// }


fn get_timestamp_ns() -> u128 {
    Instant::now().elapsed().as_nanos()
}
fn get_timestamp_ms() -> String {
    let now = Local::now();
    // 获取日期时间部分
    let datetime_str = now.format("%Y-%m-%d %H:%M:%S").to_string();
    // 获取毫秒部分
    let millis = now.timestamp_subsec_millis();
    format!("{} - {}", datetime_str, millis)
}

fn get_timestamp() -> u128 {
    // 这里仅为示例，实际应返回系统当前时间的毫秒数
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()
}

// fn main() {
//     let count = 200;

//     let dispatcher = Arc::new(Mutex::new(QueueEventDispatcher::new(count)));
    
//     let latency_stats = Arc::new(Mutex::new((0u128, 0u128)));
//     // 注册 "Trade" 事件回调
//     dispatcher.lock().unwrap().register(event::EventType::Trade, Box::new(move |event: &EventData| {
//         let callback_timestamp = get_timestamp();
//         let current = get_timestamp_ms();
//         // println!(
//         //     "[{} ms] 📢 回调触发 - 事件: Trade, 数据: {:?}",
//         //     current, event.data[0]
//         // );
//         let ts: u128 = event.data[0].parse().expect("转换失败");
//         let latency = callback_timestamp - ts;
            
//         let mut stats = latency_stats.lock().unwrap();
//         stats.0 += latency;
//         stats.1 += 1;
//         let avg_latency = stats.0 as f64 / stats.1 as f64;
//         println!(
//             "[{} ms] 📢 回调触发 - 事件: Trade, 数据: {:?}, 单次耗时: {} ms, 实时平均耗时: {:.2} ms",
//             current,
//             event.data,
//             latency,
//             avg_latency
//         );
//     }));

//     // **创建一个线程用于持续写入事件**
//     // **启动子线程，持续写入事件**
    
//     let dispatcher_clone = Arc::clone(&dispatcher);
//     thread::spawn(move || {
//         for i in 1..=count {
//             let timestamp = get_timestamp();
//             let current = get_timestamp_ms();
//             // let data = vec![format!("TradeData{}", timestamp)];
//             let data = vec![timestamp];
//             println!(
//                 "[{} ms] 📝 事件写入 - Trade, 数据: {:?}",
//                 current, data
//             );
//             dispatcher_clone.lock().unwrap().fire(event::EventType::Trade, data);
//             thread::sleep(Duration::from_millis(100)); // 每 100ms 写入一次
//         }
//     });

//     // **主线程持续 `process()`**
//     loop {
//         dispatcher.lock().unwrap().process();
//         thread::sleep(Duration::from_millis(50)); // 50ms 处理一次队列
//     }

//     println!("✅ 测试完成！");
// }

fn main() {
    let count = 200;
    let mut dispatcher = AsyncQueueEventDispatcher::new(count);
    
    let latency_stats = Arc::new(Mutex::new((0u128, 0u128)));
    // 注册 "Trade" 事件回调
    dispatcher.register(event::EventType::Trade, Box::new(move  |event: &EventData| {
        // let timestamp = get_timestamp();
        let callback_timestamp = get_timestamp();
        let current = get_timestamp_ms();
        // println!(
        //     "[{} ms] 📢 回调触发 - 事件: Trade, 数据: {:?}",
        //     current, event.data[0]
        // );
        let ts: u128 = event.data[0].parse().expect("转换失败");
        let latency = callback_timestamp - ts;
            
        let mut stats = latency_stats.lock().unwrap();
        stats.0 += latency;
        stats.1 += 1;
        let avg_latency = stats.0 as f64 / stats.1 as f64;
        println!(
            "[{} ms] 📢 回调触发 - 事件: Trade, 数据: {:?}, 单次耗时: {} ms, 实时平均耗时: {:.2} ms",
            current,
            event.data,
            latency,
            avg_latency
        );
            
    }));

    let (mut producer, mut consumer) = dispatcher.split();

    // **创建一个线程用于持续写入事件**
    // **启动子线程，持续写入事件**
    
    // let dispatcher_clone = Arc::clone(&dispatcher);
    thread::spawn(move || {
        for i in 1..=count {
            let timestamp = get_timestamp();
            let current = get_timestamp_ms();
            // let data = vec![format!("TradeData{}", timestamp)];
            let data = vec![timestamp];
            println!(
                "[{} ms] 📝 事件写入 - Trade, 数据: {:?}",
                current, data
            );
            producer.fire(event::EventType::Trade, data);
            thread::sleep(Duration::from_millis(100)); // 每 100ms 写入一次
        }
    });

    // **主线程持续 `process()`**
    loop {
        consumer.process();
        thread::sleep(Duration::from_millis(50)); // 50ms 处理一次队列
    }

    println!("✅ 测试完成！");
}