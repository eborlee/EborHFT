use std::collections::HashMap;
use ringbuf::{RingBuffer, Producer, Consumer};


use crate::event::EventType;

#[derive(Debug)]
pub struct EventData {
    pub event_type: EventType,
    pub data: Vec<String>,
}



pub trait EventDispatcher {
    // fn dispatch(&self, event: &Event);
    fn register(&mut self, event_type: EventType, call_back: Box<dyn Fn(&EventData)+ Send + Sync>);
    fn unregister(&mut self, event_type: EventType);
    fn clear_events(&mut self);

    fn m_trigger(&self, event: EventData);
}

pub struct QueueEventDispatcher {
    event_map: HashMap<EventType, Vec<Box<dyn Fn(&EventData)+ Send + Sync >>>,
    producer: Producer<EventData>, // 生产者（写入数据）
    event_queue: Consumer<EventData>,    // 消费者（读取数据）
}

impl QueueEventDispatcher {
    pub fn new(capacity: usize) -> Self {
        let event_map = HashMap::new(); // 正确声明 event_map 变量
        let rb = RingBuffer::<EventData>::new(capacity);

        let (producer, consumer) = rb.split(); // 拆分成生产者和消费者
        Self {
            producer,
            event_queue: consumer,
            event_map, // 初始化 event_map 字段
        }
    }

    // 事件入队（不带参数）
    fn enqueue(&mut self, data: EventData) {
        if self.producer.is_full() {
            println!("Warning: Event queue is full. The oldest event will be overwritten.");
            self.event_queue.pop(); // 丢弃最早的事件
        }
        let _ = self.producer.push(data);
    }

    // 事件入队（带参数）
    fn enqueue_with_args<U: ToString>(&mut self, event_type: EventType, args: Vec<U>) {
        let data = EventData {
            event_type,
            data: args.into_iter().map(|arg| arg.to_string()).collect(),
        };
        self.enqueue(data);
    }

    pub fn process(&mut self) {
        while let Some(event) = self.event_queue.pop() {
            self.m_trigger(event);
        }
    }

    pub fn fire<U: ToString>(&mut self, event_type: EventType, data: Vec<U>) {
        self.enqueue_with_args(event_type, data);
    }



    
}

impl EventDispatcher for QueueEventDispatcher {
    fn register(&mut self, event_type: EventType, call_back: Box<dyn Fn(&EventData)+ Send + Sync>) {
        let call_backs = self.event_map.entry(event_type).or_insert(Vec::new());
        call_backs.push(call_back);
    }

    fn unregister(&mut self, event_type: EventType) {
        self.event_map.remove(&event_type);
    }

    fn clear_events(&mut self) {
        self.event_map.clear();
    }

    fn m_trigger(&self, event: EventData) {
        if let Some(call_backs) = self.event_map.get(&event.event_type) {
            for call_back in call_backs {
                call_back(&event);
            }
        }
    }
}

pub struct AsyncQueueEventDispatcher {
    m_inner: QueueEventDispatcher,
}

impl EventDispatcher for AsyncQueueEventDispatcher {
    fn register(&mut self, event_type: EventType, call_back: Box<dyn Fn(&EventData)+ Send + Sync>) {
        self.m_inner.register(event_type, call_back);
    }

    fn unregister(&mut self, event_type: EventType) {
        self.m_inner.unregister(event_type);
    }

    fn clear_events(&mut self) {
        self.m_inner.clear_events();
    }

    fn m_trigger(&self, event: EventData) {
        self.m_inner.m_trigger(event);
    }
}

impl AsyncQueueEventDispatcher {
    pub fn new(capacity: usize) -> Self {
        Self {
            m_inner: QueueEventDispatcher::new(capacity),
        }
    }

    pub fn split(self) -> (QueueEventDispatcherProducer, QueueEventDispatcherConsumer) {
        (
            QueueEventDispatcherProducer {producer: self.m_inner.producer },
            QueueEventDispatcherConsumer { 
                event_queue: self.m_inner.event_queue, 
                event_map: self.m_inner.event_map,
            },
        )
    }
}

pub struct QueueEventDispatcherProducer {
    producer: Producer<EventData>,
}

pub struct QueueEventDispatcherConsumer {
    event_queue: Consumer<EventData>,
    event_map: HashMap<EventType, Vec<Box<dyn Fn(&EventData)+ Send + Sync >>>,
}

impl QueueEventDispatcherProducer {


    fn enqueue(&mut self, data: EventData) {
        let _ = self.producer.push(data);
    }

    // 事件入队（带参数）
    fn enqueue_with_args<U: ToString>(&mut self, event_type: EventType, args: Vec<U>) {
        let data = EventData {
            event_type,
            data: args.into_iter().map(|arg| arg.to_string()).collect(),
        };
        self.enqueue(data);
    }

    pub fn fire<U: ToString>(&mut self, event_type: EventType, data: Vec<U>) {
        self.enqueue_with_args(event_type, data);
    }
}

impl QueueEventDispatcherConsumer {


    fn m_trigger(&self, event: EventData) {
        if let Some(call_backs) = self.event_map.get(&event.event_type) {
            for call_back in call_backs {
                call_back(&event);
            }
        }
    }

    pub fn process(&mut self) {
        while let Some(event) = self.event_queue.pop() {
            self.m_trigger(event);
        }
    }
}