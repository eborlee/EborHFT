use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex, RwLock};
use event_engine::event::AggTradeEvent;


pub type WatchedQtySet = Arc<RwLock<HashMap<String, HashSet<String>>>>;
pub type TradeHistory = Arc<Mutex<HashMap<String, HashMap<String, VecDeque<AggTradeEvent>>>>>;

// pub fn default_watched_quantities() -> WatchedQtySet {
//     Arc::new([5.023, 10.002, 1.234].into_iter().collect())
// }
