//! 把全树排出来的盘打成一个指纹，供 `scripts/profile-parity.sh` 在两种编译档下各取一次。
//!
//! 这支程序自己不判对错——它只负责量。判在脚本那边：同一份输入、两种优化档，
//! 指纹必须一模一样。分开是有意的：一个只会打印的测试是空断言，而这里的空
//! 正是它的本分。

use mingli_contract::{Gender, Query};
use mingli_engine::cast_all;
use mingli_registry::registry;

/// 覆盖面：六个时刻横跨 1901–2050，含立春交界与午夜前后各一。
const MOMENTS: [(i32, u32, u32, u32, u32); 6] = [
    (1990, 6, 15, 14, 30),
    (1987, 9, 17, 12, 0),
    (2024, 2, 4, 0, 1),
    (2000, 1, 1, 23, 59),
    (2050, 12, 31, 6, 6),
    (1901, 3, 21, 18, 45),
];

fn main() {
    let mut blob = String::new();
    for (year, month, day, hour, minute) in MOMENTS {
        let mut query = Query::at(year, month, day, hour, minute, 8.0);
        query.gender = Some(Gender::Male);
        query.seed = Some(20_260_825);
        let all = cast_all(&registry(), &query);
        blob.push_str(&serde_json::to_string(&all).expect("盘可序列化"));
        blob.push('\n');
    }
    // FNV-1a：够用，且不引第三方依赖。
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in blob.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    println!("bytes={} fingerprint={hash:016x}", blob.len());
}
