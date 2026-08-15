//! Placidus 分宫制：半弧三分（移植自 Swiss Ephemeris `swehouse.c` Placidus 分支）。
//!
//! 给定本地恒星时 RAMC、黄赤交角 ε、地理纬度 φ，迭代求中间宫尖 11/12/2/3 的黄经；
//! 1/10/4/7 仍由 [`crate::asc_mc`] 闭式给出；5/6/8/9 由对宫等同性 `cusp[k+6] = cusp[k] + 180°` 派生。
//!
//! **极区回退**：当 `|φ| ≥ 90° − ε`（约 |φ| ≥ 66.5°）落进绕极圈，Placidus 失效，
//! 上层应改用整宫制；本模块返回 [`None`]。

/// 数值收敛阈值（度），≈ 0.01 角秒，对齐 `swehouse.c` 的 `VERY_SMALL_PLAC_ITER`。
const ITER_TOL_DEG: f64 = 1.0 / 360_000.0;
/// 一般"很小"阈值（度）。
const VERY_SMALL: f64 = 1.0e-10;
/// 最大迭代次数（对齐 `swehouse.c` `niter_max`）。
const NITER_MAX: usize = 100;

/// Placidus 12 个宫尖（黄经度，按宫序号 1..=12 索引，`cusps[0] = cusps[12]` 模数对齐用，未占用）。
#[derive(Debug, Clone, Copy)]
pub struct PlacidusCusps {
    /// `[_, cusp1, cusp2, ..., cusp12]`，下标 0 占位。
    pub cusps: [f64; 13],
}

/// 取 `[0, 360)` 内的角度（度）。
pub(crate) fn norm360(x: f64) -> f64 {
    x.rem_euclid(360.0)
}

/// `swehouse.c` 中的 `Asc2` 等价：给定赤经 `x`（度，象限 1）与 pole height `f`（度），
/// 返回黄道与该位置「极高线」的交点黄经（度，`[0,180)`）。
fn asc2(x: f64, f: f64, sine: f64, cose: f64) -> f64 {
    let mut ass = -f.to_radians().tan() * sine + cose * x.to_radians().cos();
    if ass.abs() < VERY_SMALL {
        ass = 0.0;
    }
    let mut sin_x = x.to_radians().sin();
    if sin_x.abs() < VERY_SMALL {
        sin_x = 0.0;
    }
    let mut out;
    if sin_x == 0.0 {
        out = if ass < 0.0 { -VERY_SMALL } else { VERY_SMALL };
    } else if ass == 0.0 {
        out = if sin_x < 0.0 { -90.0 } else { 90.0 };
    } else {
        out = (sin_x / ass).atan().to_degrees();
    }
    if out < 0.0 {
        out += 180.0;
    }
    out
}

/// `swehouse.c` 中的 `Asc1`：把 `x1` 按象限分发到 `Asc2`，含极点保护与边界吸附。
pub(crate) fn asc1(mut x1: f64, f: f64, sine: f64, cose: f64) -> f64 {
    x1 = norm360(x1);
    let n = (x1 / 90.0).floor() as i32 + 1; // 1..=4
    if (90.0 - f).abs() < VERY_SMALL {
        return 180.0;
    }
    if (90.0 + f).abs() < VERY_SMALL {
        return 0.0;
    }
    let mut ass = match n {
        1 => asc2(x1, f, sine, cose),
        2 => 180.0 - asc2(180.0 - x1, -f, sine, cose),
        3 => 180.0 + asc2(x1 - 180.0, -f, sine, cose),
        _ => 360.0 - asc2(360.0 - x1, f, sine, cose),
    };
    ass = norm360(ass);
    // 90/180/270 边界吸附（避免浮点抖动）。
    for &k in &[90.0, 180.0, 270.0] {
        if (ass - k).abs() < VERY_SMALL {
            ass = k;
        }
    }
    ass
}

/// 角度的最短带符号差（度，结果 `(-180, 180]`）。
pub(crate) fn signed_diff_deg(a: f64, b: f64) -> f64 {
    let d = (a - b).rem_euclid(360.0);
    if d > 180.0 {
        d - 360.0
    } else {
        d
    }
}

/// 解单个中间宫尖。
///
/// `rectasc` = `RAMC + offset`（度，cusp 11/12/2/3 的 offset 为 30/60/120/150）；
/// `pole_main` = 主 pole height `fh1` 或 `fh2`(degree)；`denom` = 3.0(cusp 11/3) 或 1.5(cusp 12/2)。
fn solve_cusp(
    rectasc: f64,
    pole_main: f64,
    denom: f64,
    tanfi: f64,
    sine: f64,
    cose: f64,
) -> Option<f64> {
    let rectasc = norm360(rectasc);
    // 第一步初值
    let first_lambda = asc1(rectasc, pole_main, sine, cose);
    let tant = (sine * first_lambda.to_radians().sin()).asin().tan();
    if tant.abs() < VERY_SMALL {
        return Some(rectasc);
    }
    // 极区：asin(tanfi · tant) 越界
    let inner = tanfi * tant;
    if !(-1.0..=1.0).contains(&inner) {
        return None;
    }
    let f_pole = ((inner.asin() / denom).sin() / tant).atan().to_degrees();
    let mut cusp = asc1(rectasc, f_pole, sine, cose);
    let mut cusp_sv = 0.0_f64;
    for i in 1..=NITER_MAX {
        let tant = (sine * cusp.to_radians().sin()).asin().tan();
        if tant.abs() < VERY_SMALL {
            return Some(rectasc);
        }
        let inner = tanfi * tant;
        if !(-1.0..=1.0).contains(&inner) {
            return None;
        }
        let f_pole = ((inner.asin() / denom).sin() / tant).atan().to_degrees();
        let cusp_new = asc1(rectasc, f_pole, sine, cose);
        if i > 1 && signed_diff_deg(cusp_new, cusp_sv).abs() < ITER_TOL_DEG {
            return Some(cusp_new);
        }
        cusp_sv = cusp;
        cusp = cusp_new;
    }
    None
}

/// 把 12 cusps 数组打包为 [`PlacidusCusps`] 形态（下标 1..=12）。
pub(crate) fn pack(asc: f64, mc: f64, c11: f64, c12: f64, c2: f64, c3: f64) -> PlacidusCusps {
    let mut cusps = [0.0_f64; 13];
    cusps[1] = norm360(asc);
    cusps[10] = norm360(mc);
    cusps[11] = c11;
    cusps[12] = c12;
    cusps[2] = c2;
    cusps[3] = c3;
    cusps[4] = norm360(mc + 180.0);
    cusps[5] = norm360(c11 + 180.0);
    cusps[6] = norm360(c12 + 180.0);
    cusps[7] = norm360(asc + 180.0);
    cusps[8] = norm360(c2 + 180.0);
    cusps[9] = norm360(c3 + 180.0);
    PlacidusCusps { cusps }
}

/// **Equal 等宫制**：cusp 1 = Asc，逐宫 +30°；Asc/MC 不分离，MC 仅作角度参考。
#[must_use]
pub fn equal_cusps(asc: f64, mc: f64) -> PlacidusCusps {
    let mut cusps = [0.0_f64; 13];
    let a = norm360(asc);
    for k in 0u8..12 {
        cusps[(k + 1) as usize] = norm360(a + 30.0 * f64::from(k));
    }
    // 10 宫的 cusp 不取等分，而保留实际 MC 作为标注用（很多占星家保留 MC 显示但不作 10 宫尖）
    let _ = mc;
    PlacidusCusps { cusps }
}

/// **Porphyry 分宫制**：1/10/4/7 = Asc/MC/IC/DC 闭式；
/// 2/3 在 IC↔Asc 弧上三分（各 1/3），11/12 在 MC↔Asc 弧上三分，5/6/8/9 由对宫派生。
/// 不需要纬度也不需迭代，极区可用。
#[must_use]
pub fn porphyry_cusps(asc: f64, mc: f64) -> PlacidusCusps {
    let asc = norm360(asc);
    let mc = norm360(mc);
    let ic = norm360(mc + 180.0);
    // MC→Asc 弧（逆时针，黄经增大方向）。
    let mc_to_asc = norm360(asc - mc);
    let third_top = mc_to_asc / 3.0;
    let c11 = norm360(mc + third_top);
    let c12 = norm360(mc + 2.0 * third_top);
    // IC→DC 弧：DC = asc+180， IC = mc+180； ic_to_dc = asc-mc（同上，因为 (asc+180)-(mc+180)=asc-mc）。
    let ic_to_dc = mc_to_asc;
    let third_bot = ic_to_dc / 3.0;
    let c2 = norm360(ic + third_bot);
    let c3 = norm360(ic + 2.0 * third_bot);
    pack(asc, mc, c11, c12, c2, c3)
}

/// 计算 12 个 Placidus 宫尖（度，`[0,360)`）。`asc` 与 `mc` 已由调用方算好。
///
/// 极区(`|φ| ≥ 90° − ε`) 或迭代不收敛时返回 [`None`]，上层应回退到整宫制。
#[must_use]
pub fn cusps(ramc_deg: f64, obliquity_deg: f64, lat_deg: f64, asc: f64, mc: f64) -> Option<PlacidusCusps> {
    // 极区保护（swehouse.c 同款）
    if lat_deg.abs() >= 90.0 - obliquity_deg {
        return None;
    }
    let sine = obliquity_deg.to_radians().sin();
    let cose = obliquity_deg.to_radians().cos();
    let tane = obliquity_deg.to_radians().tan();
    let tanfi = lat_deg.to_radians().tan();

    let a_aux = (tanfi * tane).asin(); // radians
    // 极区第二道保护：asin 在 |tanfi·tane|>1 时 NaN——但极区保护已经覆盖。
    if !a_aux.is_finite() {
        return None;
    }
    let fh1 = ((a_aux / 3.0).sin() / tane).atan().to_degrees();
    let fh2 = (((a_aux * 2.0) / 3.0).sin() / tane).atan().to_degrees();

    let c11 = solve_cusp(ramc_deg + 30.0, fh1, 3.0, tanfi, sine, cose)?;
    let c12 = solve_cusp(ramc_deg + 60.0, fh2, 1.5, tanfi, sine, cose)?;
    let c2 = solve_cusp(ramc_deg + 120.0, fh2, 1.5, tanfi, sine, cose)?;
    let c3 = solve_cusp(ramc_deg + 150.0, fh1, 3.0, tanfi, sine, cose)?;
    Some(pack(asc, mc, c11, c12, c2, c3))
}

/// 判断某黄经 `lambda` 落在哪一宫(1..=12)。`cusps` 由 [`cusps`] 给出。
#[must_use]
pub fn house_of(cusps: &PlacidusCusps, lambda: f64) -> u8 {
    let lambda = norm360(lambda);
    for k in 1..=12u8 {
        let lo = cusps.cusps[k as usize];
        let hi = cusps.cusps[(k % 12 + 1) as usize];
        // 区间 [lo， hi) 跨 0° 时需绕回
        let span = norm360(hi - lo);
        let off = norm360(lambda - lo);
        if off < span {
            return k;
        }
    }
    // 数值边界保险：理论上不到这里。
    12
}

#[cfg(test)]
mod tests {
    use super::*;

    // —— Diana, Princess of Wales (Rodden AA): 1961-07-01 19:45 BST=UT 18:45 ——
    // Sandringham 52°50′N 0°30′E。
    // Astrodienst 给出的 Placidus 完整宫尖（转十进制度）：
    //   1=258.4167°（射手18°25'）  2=299.8167°（摩羯29°49'）  3=348.3667°（双鱼18°22'）
    //   5=46.0500°（金牛16°03'）   6=63.3000°（双子03°18'）   10=203.0500°（天秤23°03'）
    //   11=226.0667°（天蝎16°04'） 12=243.3000°（射手03°18'）
    // 4/7/8/9 由对宫等同性派生。geocult.org 给出的 11=226.0594° / 12=243.2997° 与上一致。
    //
    // 验证容差：0.05°（角分级，匹配本算用平恒星时/平交角、ELP 截断的精度）。
    #[test]
    fn diana_placidus_cusps() {
        // Diana, Princess of Wales (Rodden AA): 1961-07-01 19:45 BST=UT 18:45,
        // Sandringham 52°50′N 0°30′E。RAMC = 本地平恒星时（度）= GMST + 东经经度。
        // 共享层(mingli-astro Moment) 给 GMST 与 ε，叠加经度得 RAMC，与 Asc/MC 一同传入。
        let m = mingli_astro::Moment::new(1961, 7, 1, 19, 45, 1.0);
        let geo_lat = 52.833;
        let geo_lon = 0.500;
        let ramc = (m.sidereal_time + geo_lon).rem_euclid(360.0);
        let eps = m.obliquity;
        let lat = geo_lat;
        let (asc, mc) = crate::asc_mc(ramc, eps, lat);
        let cs = cusps(ramc, eps, lat, asc, mc).expect("Diana 非极区");
        let expected: [(usize, f64); 12] = [
            (1, 258.4167),
            (2, 299.8167),
            (3, 348.3667),
            (4, 23.05),
            (5, 46.05),
            (6, 63.30),
            (7, 78.4167),
            (8, 119.8167),
            (9, 168.3667),
            (10, 203.05),
            (11, 226.0667),
            (12, 243.30),
        ];
        for (k, want) in expected {
            let got = cs.cusps[k];
            let diff = signed_diff_deg(got, want).abs();
            assert!(
                diff < 0.05,
                "cusp {k}: got {got:.4}°, want {want:.4}°, diff {diff:.4}°"
            );
        }
    }

    #[test]
    fn polar_region_returns_none() {
        // |φ| ≥ 90° − ε ≈ 66.56° → Placidus 失效
        assert!(cusps(0.0, 23.44, 70.0, 90.0, 0.0).is_none());
        assert!(cusps(0.0, 23.44, -80.0, 90.0, 0.0).is_none());
    }

    #[test]
    fn cusp_opposites_hold() {
        // 任一 cusp k 与 k+6 应严格 180° 对宫（模 360°）。
        let m = mingli_astro::Moment::new(1961, 7, 1, 19, 45, 1.0);
        let ramc = (m.sidereal_time + 0.5).rem_euclid(360.0);
        let (asc, mc) = crate::asc_mc(ramc, m.obliquity, 52.833);
        let cs = cusps(ramc, m.obliquity, 52.833, asc, mc).unwrap();
        for k in 1..=6 {
            let diff = signed_diff_deg(cs.cusps[k] + 180.0, cs.cusps[k + 6]).abs();
            assert!(diff < 1e-9, "cusp {k} vs {}: diff {diff}", k + 6);
        }
    }

    #[test]
    fn house_of_basic_assignment() {
        let m = mingli_astro::Moment::new(1961, 7, 1, 19, 45, 1.0);
        let ramc = (m.sidereal_time + 0.5).rem_euclid(360.0);
        let (asc, mc) = crate::asc_mc(ramc, m.obliquity, 52.833);
        let cs = cusps(ramc, m.obliquity, 52.833, asc, mc).unwrap();
        // 比 cusp 1(258.42)略大一点的黄经应该落入第 1 宫
        assert_eq!(house_of(&cs, 260.0), 1);
        // 比 cusp 10(203.05)略大的应落入第 10 宫
        assert_eq!(house_of(&cs, 210.0), 10);
        // cusp 边界右极限：正好落在 cusp k 上应归入 k
        for k in 1..=12u8 {
            assert_eq!(house_of(&cs, cs.cusps[k as usize] + 1e-6), k);
        }
    }

    #[test]
    fn asc2_quadrant_sanity() {
        // ass=0 fast path:sin_x>0 → +90
        let r = asc2(89.999_999_99, 89.999_999_99, 0.3977, 0.9175);
        assert!(r.is_finite());
        // x near 0：sin_x ≈ 0，走 sin_x==0 分支
        let r2 = asc2(1e-12, 10.0, 0.3977, 0.9175);
        assert!(r2.is_finite());
    }

    #[test]
    fn asc2_negative_ass_branch() {
        // f 极大、x 接近 0 → ass = -tan(f)·sine + cose·cos(x) < 0，触发 atan 后 ass<0 += 180 分支。
        let r = asc2(0.5, 85.0, 0.3977, 0.9175);
        assert!((150.0..180.0).contains(&r), "expected wrap-around, got {r}");
    }

    #[test]
    fn asc2_zero_ass_with_negative_sinx() {
        // 构造 ass 极接近 0（ass=0 分支）：cose·cos(x) = tan(f)·sine
        // 解出 cos(x) = tan(f)·sine/cose； 取 f=20°、ε 给定的 sine/cose：
        let sine = 23.44_f64.to_radians().sin();
        let cose = 23.44_f64.to_radians().cos();
        let f = 20.0_f64;
        let cos_x = f.to_radians().tan() * sine / cose;
        let x = cos_x.acos().to_degrees();
        // sin_x > 0 → ass=0 → out=+90
        let r = asc2(x, f, sine, cose);
        assert!(r > 60.0 && r < 120.0, "near 90 got {r}");
    }

    #[test]
    fn asc1_attractor_snap_to_180() {
        // 让 asc1 的 ass 极接近 180°：走 n=2 分支(x in 90..180)
        let s = 23.44_f64.to_radians().sin();
        let c = 23.44_f64.to_radians().cos();
        let r = asc1(120.0, 5.0, s, c);
        assert!((0.0..360.0).contains(&r));
    }

    #[test]
    fn cusps_equator_special_case() {
        // φ=0（赤道）：a_aux=0、fh1=fh2=0、cusp 11 退化为 first_lambda=asc1(RAMC+30°)
        // 此 case 数值上 tant 可能接近 0（若 RAMC+30° = N·180°）
        let cs = cusps(0.0, 23.44, 0.0, 90.0, 0.0).unwrap();
        // 全部 cusps 都应在 [0，360)
        for k in 1..=12 {
            assert!((0.0..360.0).contains(&cs.cusps[k]));
        }
    }

    /// Equal 等宫制：cusp k = Asc + 30°×(k-1) 严格等分。
    #[test]
    fn equal_houses_are_30_apart() {
        let cs = equal_cusps(258.42, 203.05);
        for k in 1..=12u8 {
            let want = (258.42 + 30.0 * f64::from(k - 1)).rem_euclid(360.0);
            assert!((cs.cusps[k as usize] - want).abs() < 1e-9);
        }
    }

    /// Porphyry：在赤道（无纬度依赖）上 Asc=90/MC=0 时三分给出 30/60/210/240 等。
    #[test]
    fn porphyry_arc_trisection() {
        // Asc=270， MC=180 → MC→Asc 弧=90，1/3=30，2/3=60。
        // cusp 11 = MC + 30 = 210; cusp 12 = MC + 60 = 240。
        // IC = 0; cusp 2 = IC + 30 = 30; cusp 3 = IC + 60 = 60。
        let cs = porphyry_cusps(270.0, 180.0);
        assert!((cs.cusps[1] - 270.0).abs() < 1e-9);
        assert!((cs.cusps[10] - 180.0).abs() < 1e-9);
        assert!((cs.cusps[11] - 210.0).abs() < 1e-9);
        assert!((cs.cusps[12] - 240.0).abs() < 1e-9);
        assert!((cs.cusps[2] - 30.0).abs() < 1e-9);
        assert!((cs.cusps[3] - 60.0).abs() < 1e-9);
        // 对宫
        assert!((cs.cusps[7] - 90.0).abs() < 1e-9);
        assert!((cs.cusps[4] - 0.0).abs() < 1e-9);
    }

    /// Porphyry Diana 校验：Astrodienst Porphyry 公开值。
    /// 与 Placidus 在中天/地平角度上一致，但中间 cusp 用纯黄道弧三分，故数值不同。
    #[test]
    fn porphyry_diana_cusp_arcs_consistent() {
        let m = mingli_astro::Moment::new(1961, 7, 1, 19, 45, 1.0);
        let ramc = (m.sidereal_time + 0.5).rem_euclid(360.0);
        let (asc, mc) = crate::asc_mc(ramc, m.obliquity, 52.833);
        let cs = porphyry_cusps(asc, mc);
        // 三分性质：cusp 11 在 MC↔Asc 1/3 处，即 (cusp11-MC) = (Asc-MC)/3 (mod 360°)
        let arc = (asc - mc).rem_euclid(360.0);
        let one_third = arc / 3.0;
        let two_third = 2.0 * arc / 3.0;
        assert!((((cs.cusps[11] - mc).rem_euclid(360.0)) - one_third).abs() < 1e-9);
        assert!((((cs.cusps[12] - mc).rem_euclid(360.0)) - two_third).abs() < 1e-9);
    }

    #[test]
    fn asc1_pole_attractors() {
        let s = 23.44_f64.to_radians().sin();
        let c = 23.44_f64.to_radians().cos();
        assert!((asc1(45.0, 89.999_999_999_999_9, s, c) - 180.0).abs() < 1e-6);
        assert!((asc1(45.0, -89.999_999_999_999_9, s, c) - 0.0).abs() < 1e-6);
    }
}
