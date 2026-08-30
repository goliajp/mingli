//! 截断后的 VSOP87D：同一份系数，丢掉振幅低于 1e-6 的项。
//!
//! 31,498 项留 1,996（6.3%）。代价是日心黄经最多差 1.718″（1900–2100 全八星实测，
//! 见 `the_truncated_series_stays_within_this_much`），而公开盘只精确到角分——粗 35 倍。
//! 换来的是表小约十六倍、级数求值快约十四倍。
//!
//! 系数不是新写的权威值，是 `vsop87` crate 那份已验证系数的子集，所以残差对着全量量，
//! 不需要第二个来源。表由 `scripts/gen-eph-lite.py` 生成，那个脚本在生成前会先证明
//! 自己的求值与 `vsop87` crate 一致。

mod earth;
mod jupiter;
mod mars;
mod mercury;
mod neptune;
mod saturn;
mod uranus;
mod venus;

use crate::Body;

/// 日心球面坐标：黄经、黄纬（弧度）与距离（AU）。
pub(crate) struct Spherical {
    pub(crate) lon: f64,
    pub(crate) lat: f64,
    pub(crate) dist: f64,
}

/// `Σ_n τ^n · Σ_i A_i·cos(B_i + C_i·τ)`——VSOP87 的求值式本身。
fn series(orders: &[&[[f64; 3]]], tau: f64) -> f64 {
    let mut total = 0.0;
    let mut power = 1.0;
    for terms in orders {
        let mut s = 0.0;
        for t in *terms {
            s += t[0] * (t[1] + t[2] * tau).cos();
        }
        total += s * power;
        power *= tau;
    }
    total
}

macro_rules! planet {
    ($m:ident) => {{
        use $m::*;
        (
            &[&L0[..], &L1[..], &L2[..], &L3[..], &L4[..], &L5[..]][..],
            &[&B0[..], &B1[..], &B2[..], &B3[..], &B4[..], &B5[..]][..],
            &[&R0[..], &R1[..], &R2[..], &R3[..], &R4[..], &R5[..]][..],
        )
    }};
}

/// 一颗天体在 `jde` 的日心球面坐标。
///
/// `Body::Sun` 与 `Body::Moon` 走地球那条（与全量实现同款约定：地心太阳 = 地球 + 180°，
/// 月亮不走日心坐标）。
pub(crate) fn heliocentric(body: Body, jde: f64) -> Spherical {
    let tau = (jde - 2_451_545.0) / 365_250.0;
    let (l, b, r) = match body {
        Body::Mercury => planet!(mercury),
        Body::Venus => planet!(venus),
        Body::Mars => planet!(mars),
        Body::Jupiter => planet!(jupiter),
        Body::Saturn => planet!(saturn),
        Body::Uranus => planet!(uranus),
        Body::Neptune => planet!(neptune),
        Body::Sun | Body::Moon => planet!(earth),
    };
    Spherical {
        lon: series(l, tau).rem_euclid(std::f64::consts::TAU),
        lat: series(b, tau),
        dist: series(r, tau),
    }
}
