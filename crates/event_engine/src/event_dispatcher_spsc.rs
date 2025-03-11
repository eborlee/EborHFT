// spsc_event_dispatcher.rs

use std::collections::HashMap;
use ringbuf::{RingBuffer, Producer, Consumer};
use crate::event::{EventType, EventPayload}; // 根据你的项目目录调整路径

use crate::event_dispatcher::{EventDispatcher, EventData};


/// SPSC 版本的事件分发器（内部使用 ringbuf）
pub struct QueueEventDispatcherSPSC {
    pub event_map: HashMap<EventType, Vec<Box<dyn Fn(&EventData) + Send + Sync>>>,
    pub producer: Producer<EventData>,
    pub event_queue: Consumer<EventData>,
}

impl QueueEventDispatcherSPSC {
    pub fn new(capacity: usize) -> Self {
        let event_map = HashMap::new();
        let rb = RingBuffer::<EventData>::new(capacity);
        let (producer, consumer) = rb.split();
        Self {
            event_map,
            producer,
            event_queue: consumer,
        }
    }

    /// 内部将事件入队
    fn enqueue(&mut self, data: EventData) {
        // 如果队列满时可考虑丢弃最旧事件或返回错误，这里忽略错误
        let _ = self.producer.push(data);
    }

    /// 主动处理队列中的所有事件
    pub fn process(&mut self) {
        while let Some(event) = self.event_queue.pop() {
            self.m_trigger(event);
        }
    }

    /// 对外接口：事件入队
    pub fn fire(&mut self, event_type: EventType, data: EventPayload) {
        let event = EventData { event_type, data };
        self.enqueue(event);
    }
}

impl EventDispatcher for QueueEventDispatcherSPSC {
    fn register(&mut self, event_type: EventType, call_back: Box<dyn Fn(&EventData) + Send + Sync>) {
        let callbacks = self.event_map.entry(event_type).or_insert(Vec::new());
        callbacks.push(call_back);
    }

    fn unregister(&mut self, event_type: EventType) {
        self.event_map.remove(&event_type);
    }

    fn clear_events(&mut self) {
        self.event_map.clear();
    }

    fn m_trigger(&self, event: EventData) {
        if let Some(callbacks) = self.event_map.get(&event.event_type) {
            for cb in callbacks {
                cb(&event);
            }
        }
    }
}

/// 可选：包装成异步版本，提供 Producer/Consumer 分离接口
pub struct AsyncQueueEventDispatcherSPSC {
    m_inner: QueueEventDispatcherSPSC,
}

impl AsyncQueueEventDispatcherSPSC {
    pub fn new(capacity: usize) -> Self {
        Self {
            m_inner: QueueEventDispatcherSPSC::new(capacity),
        }
    }

    /// 分拆出 Producer 和 Consumer 对象
    pub fn split(self) -> (QueueEventDispatcherSPSCProducer, QueueEventDispatcherSPSCConsumer) {
        (
            QueueEventDispatcherSPSCProducer { producer: self.m_inner.producer },
            QueueEventDispatcherSPSCConsumer { 
                event_queue: self.m_inner.event_queue, 
                event_map: self.m_inner.event_map,
            },
        )
    }
}

/// Producer 对象
pub struct QueueEventDispatcherSPSCProducer {
    pub producer: Producer<EventData>,
}

impl QueueEventDispatcherSPSCProducer {
    pub fn fire(&mut self, event_type: EventType, data: EventPayload) {
        let event = EventData { event_type, data };
        let _ = self.producer.push(event);
    }
}

/// Consumer 对象
pub struct QueueEventDispatcherSPSCConsumer {
    pub event_queue: Consumer<EventData>,
    pub event_map: HashMap<EventType, Vec<Box<dyn Fn(&EventData) + Send + Sync>>>,
}

impl QueueEventDispatcherSPSCConsumer {
    /// 处理所有事件，触发注册的回调
    pub fn process(&mut self) {
        while let Some(event) = self.event_queue.pop() {
            if let Some(callbacks) = self.event_map.get(&event.event_type) {
                for cb in callbacks {
                    cb(&event);
                }
            }
        }
    }
}
