//! 打印 `vsop87` crate 的原始 L/B/R，供 `scripts/gen-eph-lite.py` 自检用。
//!
//! 生成截断表之前要先证明生成器自己的求值是对的；这个 example 就是那把尺子。
fn main() {
    use vsop87::vsop87d;
    /// 名字与它那条级数。
    type Series = fn(f64) -> vsop87::SphericalCoordinates;
    let fs: [(&str, Series); 8] = [
        ("mercury", vsop87d::mercury), ("venus", vsop87d::venus), ("earth", vsop87d::earth),
        ("mars", vsop87d::mars), ("jupiter", vsop87d::jupiter), ("saturn", vsop87d::saturn),
        ("uranus", vsop87d::uranus), ("neptune", vsop87d::neptune),
    ];
    for (name, f) in fs {
        for i in 0..8 {
            let jde = 2_415_020.0 + f64::from(i) * 10_400.0;
            let c = f(jde);
            println!("{name} {jde} {:.14} {:.14} {:.14}", c.longitude(), c.latitude(), c.distance());
        }
    }
}
