// mpsc_event_dispatcher.rs

use std::collections::HashMap;
use crossbeam_channel::{bounded, Sender, Receiver};
use crate::event::{EventType, EventPayload}; 
use crate::event_dispatcher::{EventDispatcher, EventData};


/// MPSC 版本的事件分发器（内部使用 crossbeam_channel）
pub struct QueueEventDispatcherMPSC {
    pub event_map: HashMap<EventType, Vec<Box<dyn Fn(&EventData) + Send + Sync>>>,
    pub sender: Sender<EventData>,
    pub receiver: Receiver<EventData>,
}

impl QueueEventDispatcherMPSC {
    pub fn new(capacity: usize) -> Self {
        let event_map = HashMap::new();
        let (sender, receiver) = bounded(capacity);
        Self {
            event_map,
            sender,
            receiver,
        }
    }

    /// 内部将事件入队
    fn enqueue(&self, data: EventData) {
        let _ = self.sender.send(data).unwrap();
    }

    /// 主动处理队列中的所有事件
    pub fn process(&self) {
        // 使用 try_recv 避免阻塞
        while let Ok(event) = self.receiver.try_recv() {
            self.m_trigger(event);
        }
    }

    /// 对外接口：事件入队
    pub fn fire(&self, event_type: EventType, data: EventPayload) {
        let event = EventData { event_type, data };
        self.enqueue(event);
    }
}

impl EventDispatcher for QueueEventDispatcherMPSC {
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
pub struct AsyncQueueEventDispatcherMPSC {
    m_inner: QueueEventDispatcherMPSC,
}

impl AsyncQueueEventDispatcherMPSC {
    pub fn new(capacity: usize) -> Self {
        Self {
            m_inner: QueueEventDispatcherMPSC::new(capacity),
        }
    }

    /// 分拆出 Producer 和 Consumer 对象
    pub fn split(self) -> (QueueEventDispatcherMPSCProducer, QueueEventDispatcherMPSCConsumer) {
        (
            QueueEventDispatcherMPSCProducer { sender: self.m_inner.sender },
            QueueEventDispatcherMPSCConsumer { 
                receiver: self.m_inner.receiver, 
                event_map: self.m_inner.event_map,
            },
        )
    }
}

/// Producer 对象
pub struct QueueEventDispatcherMPSCProducer {
    pub sender: Sender<EventData>,
}

impl QueueEventDispatcherMPSCProducer {
    pub fn fire(&self, event_type: EventType, data: EventPayload) {
        let event = EventData { event_type, data };
        let _ = self.sender.send(event).unwrap();
    }
}

/// Consumer 对象
pub struct QueueEventDispatcherMPSCConsumer {
    pub receiver: Receiver<EventData>,
    pub event_map: HashMap<EventType, Vec<Box<dyn Fn(&EventData) + Send + Sync>>>,
}

impl QueueEventDispatcherMPSCConsumer {
    /// 处理所有事件，触发注册的回调
    pub fn process(&self) {
        while let Ok(event) = self.receiver.try_recv() {
            if let Some(callbacks) = self.event_map.get(&event.event_type) {
                for cb in callbacks {
                    cb(&event);
                }
            }
        }
    }
}
