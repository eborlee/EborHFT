// use std::time::{SystemTime, UNIX_EPOCH, Instant};

// fn main() {
//     let start = Instant::now();
//     for _ in 0..1_000_000 {
//         let _ = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros();
//     }
//     let elapsed = start.elapsed();
//     println!("Average per call: {} ns", elapsed.as_nanos() / 1_000_000);
// }

use simd_json::serde::from_slice;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct MyStruct {
    name: String,
    age: u32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let json_str = r#"{"name":"Alice","age":30}"#;
    
    // 正确做法：分配 buffer 并添加 padding
    let mut data = json_str.as_bytes().to_vec();
    data.resize(data.len() +128, 0);

    let parsed: MyStruct = from_slice(&mut data)?;
    println!("{:?}", parsed);

    Ok(())
}
