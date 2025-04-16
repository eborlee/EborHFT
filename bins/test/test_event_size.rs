use std::mem::{size_of, align_of};
use std::ptr;
use std::mem::offset_of;

#[repr(C, align(64))]
#[derive(Debug, Clone, Copy)]
pub struct AggTradeEvent {
    pub price: f64,               // 8
    pub quantity: f64,            // 8
    pub trade_time: u64,          // 8
    pub event_time: u64,          // 8
    pub received_timestamp: u64,  // 8
    pub agg_trade_id: u64,        // 8
    pub symbol: [u8; 12],         // 12
    pub is_buyer_maker: bool,     // 1
    pub event_type: u8,          // 1
    pub _pad: [u8; 2],            // 3 ← 手动补 3 字节凑满 64
}

fn main() {
    println!("AggTradeEvent size: {} bytes", size_of::<AggTradeEvent>());
    println!("AggTradeEvent align: {} bytes", align_of::<AggTradeEvent>());

    println!("\nField Offsets:");
    println!(
        "  price               @ {}",
        offset_of!(AggTradeEvent, price)
    );
    println!(
        "  quantity            @ {}",
        offset_of!(AggTradeEvent, quantity)
    );
    println!(
        "  trade_time          @ {}",
        offset_of!(AggTradeEvent, trade_time)
    );
    println!(
        "  event_time          @ {}",
        offset_of!(AggTradeEvent, event_time)
    );
    println!(
        "  agg_trade_id        @ {}",
        offset_of!(AggTradeEvent, agg_trade_id)
    );
    println!(
        "  symbol              @ {}",
        offset_of!(AggTradeEvent, symbol)
    );
    println!(
        "  is_buyer_maker      @ {}",
        offset_of!(AggTradeEvent, is_buyer_maker)
    );
    println!(
        "  event_type          @ {}",
        offset_of!(AggTradeEvent, event_type)
    );
    println!(
        "  received_timestamp  @ {}",
        offset_of!(AggTradeEvent, received_timestamp)
    );
    println!(
        "  _pad                @ {}",
        offset_of!(AggTradeEvent, _pad)
    );
}

// Helper macro to calculate field offset using unsafe
macro_rules! offset_of {
    ($type:ty, $field:ident) => {
        unsafe { 
            let instance = ptr::null::<$type>();
            let offset = &(*instance).$field as *const _ as usize;
            offset
        }
    };
}
