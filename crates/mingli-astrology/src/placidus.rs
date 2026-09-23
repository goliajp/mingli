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
        // 这个 `<` 松成 `<=` 会把结果从 ~0 翻到 180°——两支同时退化时它是唯一的分水岭。
        // 由 `asc1_is_pinned_branch_by_branch` 里 `f = 90 − ε` 那几格钉住。
        out = if ass < 0.0 { -VERY_SMALL } else { VERY_SMALL };
    } else if ass == 0.0 {
        // `sin_x < 0.0` 在本函数的契约下走不到：`asc1` 只用象限一的 x（0..=90），
        // 那里 sin 非负。变异扫描会把这一支的比较与那个负号列成漏网，
        // 它们是**走不到的代码**上的变异，不是守卫的缺口。保留是为了跟 swehouse.c 逐行对得上。
        out = if sin_x < 0.0 { -90.0 } else { 90.0 };
    } else {
        out = (sin_x / ass).atan().to_degrees();
    }
    // 这里的 `<` 松成 `<=` 是等价变异：`out` 取不到 0——`sin_x == 0` 那支给 ±VERY_SMALL，
    // 另外两支给 ±90 或一个 atan 的非零值。
    if out < 0.0 {
        out += 180.0;
    }
    out
}

/// `swehouse.c` 中的 `Asc1`：把 `x1` 按象限分发到 `Asc2`，含极点保护与边界吸附。
pub(crate) fn asc1(mut x1: f64, f: f64, sine: f64, cose: f64) -> f64 {
    x1 = norm360(x1);
    let n = (x1 / 90.0).floor() as i32 + 1; // 1..=4
    // 这两条是极点的**捷径**，不是分岔：`f → ±90` 时 `tan f` 发散，下面的通式本来
    // 就趋向同一个 180 / 0。所以把它们的比较放宽、甚至把 `90.0 - f` 写成 `90.0 / f`，
    // 结果只差 1e-14 量级——变异扫描列出的那几条是近似等价，不是没人守。
    // 由 `the_pole_guards_short_circuit_before_the_quadrants` 钉住它们确实短路。
    if (90.0 - f).abs() < VERY_SMALL {
        return 180.0;
    }
    if (90.0 + f).abs() < VERY_SMALL {
        return 0.0;
    }
    // 第三象限那一支与第四象限那条**恒等**：把 asc2 的定义代进去可得
    // `asc2(180 − u, f) = 180 − asc2(u, −f)`，于是 `180 + asc2(x−180, −f)`
    // 与 `360 − asc2(360 − x, f)` 处处相同（实测差 0 与 5.7e-14）。
    // 所以变异扫描把「删掉第三支」与「`x1 - 180` 改成 `x1 + 180`」（后者模 360 同值）
    // 报成漏网，两条都是等价变异。保留三支是为了跟 swehouse.c 的分发一一对应。
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
/// 回代残差为什么不是那把钥匙（2026-08-28 试过并撤回）。
///
/// 公开实测盘只精确到角分，而这里收敛到百分之一角秒，凡位移不足一角分的改动都拦不住——
/// 加第二张盘（爱因斯坦）只多杀三个变异体。顺理成章的下一步是拿解回代它所解的方程：
/// 由黄经取赤纬正切，以 `asin(tanφ·tant)/n` 定极高，再由 `asc1` 从赤经读回黄经，
/// 收敛意味着读回它自己。
///
/// 实测：多杀 1 个，却把 13 个原本被抓的变异体拖成超时，扫描时间也涨。原因是那个
/// 「另写一遍」的映射里调用了 `asc1`／`asc2`，而多数存活变异体正住在那两个函数里——
/// 两边一起被改，残差抵消。要成为真正的独立对照，得连 `asc1`/`asc2` 一并在测试里重写。
/// 撤回了。剩下的存活多半仍是低于分辨率的那一类。
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
    // 这个 `<` 与循环里那个同款的，松成 `<=` 都是等价变异：差别只在
    // `tant.abs()` 恰好等于 1e-10 的那一个比特上，本域内取不到。
    if tant.abs() < VERY_SMALL {
        return Some(rectasc);
    }
    // 极区：asin(tanfi · tant) 越界
    let inner = tanfi * tant;
    if !(-1.0..=1.0).contains(&inner) {
        return None;
    }
    // 这一行与 `cusps` 里的 fh1/fh2 同理：只是迭代的**起点**，循环每轮都从收敛中的
    // cusp 重算极高。改这里的算术不改变答案，只改变迭代次数——等价变异。
    // 由 `the_iteration_forgets_the_pole_it_started_from` 钉住。
    let f_pole = ((inner.asin() / denom).sin() / tant).atan().to_degrees();
    let mut cusp = asc1(rectasc, f_pole, sine, cose);
    // 上一轮的值。头一轮没有上一轮——从前这里是个 0.0 的哨兵配一个 `i > 1`，
    // 于是「哨兵恰好像个真值」成了可能：宫尖若落在 0° 前 0.01 角秒内，
    // 把 `i > 1` 松成 `i >= 1` 就会在第一轮提前收工，而没有测试够得着那个角落。
    // 用 `Option` 说出本意后，那个比较连同它的变异体一起不存在了。
    let mut previous: Option<f64> = None;
    for _ in 1..=NITER_MAX {
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
        // 比的是隔一轮的值（不是上一轮），这样两步一循环的振荡也能收住。
        // 这里的 `<` 松成 `<=` 只在差值恰等于阈值的那一个比特上不同，是等价变异。
        if previous.is_some_and(|prev| signed_diff_deg(cusp_new, prev).abs() < ITER_TOL_DEG) {
            return Some(cusp_new);
        }
        previous = Some(cusp);
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
    // 下面六行的 `+ 180.0` 改成 `- 180.0` 不会有任何测试红，那不是缺口：
    // 模 360 之下两者恒等。变异扫描会把这六条列成漏网，它们是等价变异。
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
    let ic = norm360(mc + 180.0); // `- 180.0` 与它模 360 恒等，是等价变异
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
    // 极区保护（swehouse.c 同款）。
    //
    // 它与下面那道 `a_aux` 非有限的判据是**同一条界**：`tan φ · tan ε > 1`
    // 等价于 `|φ| > 90° − ε`。所以把这里的减号改成加号（门槛跳到 113°，形同虚设）
    // 不会有测试红——`a_aux` 那道会接住。是等价变异，不是缺口；这里留着是快路径。
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
    // fh1 / fh2 只是喂给 `solve_cusp` 的**迭代初值**——它进去只算一个 `first_lambda`，
    // 之后每一轮都从收敛中的 cusp 重算极高，收到 0.01 角秒为止。故这两行的算术
    // 改动不改变答案，只改变迭代次数：变异测试会把它们报成「没被拦住」，那是等价变异。
    // 这件事由 `the_iteration_forgets_the_pole_it_started_from` 钉住。
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

/// 直接对着数值奇点做的单元测试。
///
/// Placidus 是从 `swehouse.c` 移植的迭代解，正常盘走的是主路径；这里几条分支只在
/// 三角函数退化时才走到（`sin x = 0`、`ass = 0`、宫尖恰落 90° 倍数）。它们看不见摸不着，
/// 却决定了极端时刻的盘面，所以按输入直接钉住。
#[cfg(test)]
mod singularities {
    use super::*;

    fn eps() -> (f64, f64) {
        let e = 23.44_f64.to_radians();
        (e.sin(), e.cos())
    }

    #[test]
    fn asc2_handles_both_degenerate_directions() {
        let (sine, cose) = eps();
        // x = 0 → sin x = 0：结果吸到 ±VERY_SMALL（再由负值 +180° 归一）。
        let at_zero = asc2(0.0, 10.0, sine, cose);
        assert!(at_zero.abs() < 1e-9 || (at_zero - 180.0).abs() < 1e-9, "实得 {at_zero}");
        // ass = 0 需要 tan f · sin ε = cos ε · cos x：取 x = 90° 使 cos x = 0，
        // 再令 f = 0 使 tan f = 0，两边同为 0。此时 sin x = 1 > 0 → 90°。
        assert!((asc2(90.0, 0.0, sine, cose) - 90.0).abs() < 1e-9);
        // 对称一侧：x = 270°（sin x = −1）→ −90°，归一后 90°。
        assert!((asc2(270.0, 0.0, sine, cose) - 90.0).abs() < 1e-9);
        // ass < 0 的一侧：x = 180° 时 cos x = −1，两项同号相加为负。
        let at_pi = asc2(180.0, 10.0, sine, cose);
        assert!(at_pi.abs() < 1e-9 || (at_pi - 180.0).abs() < 1e-9, "实得 {at_pi}");
        // 常规输入落在 [0,180)
        for x in [10.0, 45.0, 123.0, 200.0, 355.0] {
            let v = asc2(x, 12.0, sine, cose);
            assert!((0.0..180.0).contains(&v), "asc2({x}) = {v} 出界");
        }
    }

    #[test]
    fn asc1_snaps_to_the_quadrant_boundaries() {
        let (sine, cose) = eps();
        // f 趋近 ±90°（极点）时有闭式出口，不进象限分发。
        assert!((asc1(37.0, 90.0, sine, cose) - 180.0).abs() < 1e-12);
        assert!(asc1(37.0, -90.0, sine, cose).abs() < 1e-12);
        // 四个象限各取一点，结果都在 [0,360) 且随 x 单调推进。
        let vs: Vec<f64> = [10.0, 100.0, 190.0, 280.0]
            .iter()
            .map(|&x| asc1(x, 20.0, sine, cose))
            .collect();
        for v in &vs {
            assert!((0.0..360.0).contains(v), "asc1 出界 {v}");
        }
        assert!(vs.windows(2).all(|w| w[0] < w[1]), "四象限应递增：{vs:?}");
        // 边界吸附：x1 使 ass 落在 90/180/270 的极小邻域内时吸到整值。
        let boundary = asc1(90.0, 0.0, sine, cose);
        assert!((boundary - 90.0).abs() < 1e-12, "实得 {boundary}");
    }

    /// `solve_cusp` 的两条早退：宫尖落在黄道交点上（tan t → 0，直接取赤经），
    /// 以及高纬下 `asin` 的定义域越界（无解，交由上层回退）。
    #[test]
    fn solving_a_cusp_bails_out_on_a_flat_tangent_or_an_out_of_domain_arcsine() {
        let (sine, cose) = eps();
        // pole = 90° 让 asc1 走极点出口给出 180°，sin 180° = 0 → tan t = 0 → 原样返回赤经。
        let flat = solve_cusp(37.0, 90.0, 3.0, 0.5, sine, cose);
        assert_eq!(flat, Some(37.0), "tan t 为零时应原样返回赤经");
        // φ = 80° 的 tan φ 乘上 tan t 越过 1，asin 无定义 → None。
        let tanfi = 80.0_f64.to_radians().tan();
        assert_eq!(solve_cusp(90.0, 0.0, 3.0, tanfi, sine, cose), None, "越界应返回 None");
        // 中纬度正常解得出来。
        let ok = solve_cusp(90.0, 0.0, 3.0, 45.0_f64.to_radians().tan(), sine, cose);
        assert!(ok.is_some_and(|v| (0.0..360.0).contains(&v)), "中纬度应有解");
    }

    #[test]
    fn the_polar_guard_fires_before_any_iteration() {
        // |φ| ≥ 90° − ε ≈ 66.56°：绕极圈内 Placidus 无解，直接 None。
        assert!(cusps(0.0, 23.44, 66.56, 90.0, 0.0).is_none(), "北极圈内应无解");
        assert!(cusps(0.0, 23.44, -66.56, 90.0, 0.0).is_none(), "南极圈内应无解");
        // 圈外一点点就该解得出来。
        assert!(cusps(0.0, 23.44, 66.0, 90.0, 0.0).is_some());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 极区门槛就在 `|φ| = 90° − ε` 上，两侧各验一次。
    ///
    /// `cusps` 用 `lat_deg.abs() >= 90.0 - obliquity_deg` 挡极区。把那个减号改成加号，
    /// 门槛从 66.56° 跳到 113.44°——再没有一个纬度够得着，保护形同虚设，而所有测试照常绿：
    /// 它们取的纬度都在温带，离门槛十几度远。
    ///
    /// 所以在门槛两侧各取一点：里面要给得出十二宫，外面要 `None`（上层据此回落 Porphyry）。
    #[test]
    fn the_polar_cutoff_sits_where_the_obliquity_puts_it() {
        const EPS: f64 = 23.44;
        let cutoff = 90.0 - EPS; // 66.56°
        for &(lat, want_some) in &[
            (cutoff - 0.5, true),
            (-(cutoff - 0.5), true),
            (cutoff + 0.5, false),
            (-(cutoff + 0.5), false),
            (89.0_f64, false),
        ] {
            let (asc, mc) = crate::asc_mc(120.0, EPS, lat);
            let got = cusps(120.0, EPS, lat, asc, mc);
            assert_eq!(
                got.is_some(),
                want_some,
                "纬度 {lat}° 在门槛 {cutoff}° 的{}侧，应{}给出宫尖",
                if want_some { "内" } else { "外" },
                if want_some { "" } else { "不" }
            );
        }
    }

    /// `asc1` 的每个分支，逐点钉住。
    ///
    /// 这里此前只有性质：绕一圈不倒退、宫尖有序。性质拦不住把算式改坏——变异扫描下
    /// `asc1` 与 `asc2` 共留二十个漏网，其中一个把**第三象限那条分支整个删掉**
    /// （`n == 3` 落到第四象限的公式上）都没有一条测试红。
    ///
    /// 取样按分支铺开：四个象限各两点、`f` 取正负（`asc2` 的第二/三象限调用会翻 `f` 的号）、
    /// 以及 `x = 0/90/180/270` 这四个退化点——`sin x == 0` 与 `ass == 0` 那两支只在那里走到，
    /// 而绕圈测试的步长恰好跨过它们。
    ///
    /// 期望值由 `swehouse.c` 的 `Asc1`/`Asc2` 算式逐点算出（那两条算式已在函数注释里引用），
    /// 钉的是转写：它答的是「有没有人改过这些分支」，不是「Placidus 对不对」——
    /// 后者由下面对 pyswisseph 十二宫尖的比对守着。
    #[test]
    fn asc1_is_pinned_branch_by_branch() {
        let (sine, cose) = (23.44_f64.to_radians().sin(), 23.44_f64.to_radians().cos());
        for &(x, f, want) in &[
            (0.0_f64, 0.0_f64, 0.000_000_000_1_f64),
            (45.0, 0.0, 47.464_329_561_9),
            (90.0, 0.0, 90.0),
            (135.0, 0.0, 132.535_670_438_1),
            (180.0, 0.0, 180.0),
            (225.0, 0.0, 227.464_329_561_9),
            (270.0, 0.0, 270.0),
            (315.0, 0.0, 312.535_670_438_1),
            (30.0, 52.0, 60.281_200_276_1),
            (120.0, 52.0, 138.179_062_254_0),
            (200.0, 52.0, 194.004_661_551_2),
            (300.0, 52.0, 266.668_825_057_6),
            (30.0, -52.0, 20.982_941_111_6),
            (120.0, -52.0, 86.668_825_057_6),
            (200.0, -52.0, 224.094_887_805_7),
            (300.0, -52.0, 318.179_062_254_0),
            (0.0, 40.0, 0.000_000_000_1),
            (180.0, 40.0, 180.0),
            (90.0, 66.0, 131.779_118_868_7),
            (270.0, -66.0, 311.779_118_868_7),
            // `f = 90 − ε` 上 `tan f · sin ε` 恰好等于 `cos ε`，于是 x=0 处 `ass` 与
            // `sin x` 同时落到零附近（实测 ass = −2.2e-16）。这一格是 `asc2` 里
            // 「两支都退化」的唯一入口：`ass` 靠上面那条吸附归零，`ass < 0.0` 才判假、
            // 才走到 `+VERY_SMALL`。吸附去掉、或那个 `<` 松成 `<=`，结果都会翻到 180°。
            (0.0, 66.56, 0.000_000_000_1),
            (180.0, -66.56, 180.0),
            (90.0, 66.56, 132.535_670_438_098),
            (270.0, 66.56, 227.464_329_561_902),
        ] {
            let got = asc1(x, f, sine, cose);
            assert!(
                (got - want).abs() < 1e-9,
                "asc1({x}, {f}) = {got}，应为 {want}——分支或算式动过了"
            );
        }
    }

    /// 边界吸附之后，落在 90/180/270 上的必须是**恰好**那个数。
    ///
    /// `asc1` 末尾那个循环把 1e-10 以内的抖动收成整数。上面那张表用的是 1e-9 的容差，
    /// 盖得住这点残差——于是把吸附条件写成 `(ass + k).abs()`（`ass` 落在 [0,360)，
    /// 那个条件永不成立，吸附形同虚设）也照样绿。
    ///
    /// 所以这里用相等而不是容差：吸附要的就是「恰好」，那就照它的本意断言。
    /// x=180、f=0 走第三象限，`180 + asc2(0, 0)` 给出 180 + VERY_SMALL，正是它该收的那种。
    #[test]
    #[allow(
        clippy::float_cmp,
        reason = "相等就是被测的性质本身：吸附要的是恰好落在整数上，容差会把它盖掉"
    )]
    fn the_boundary_snap_lands_exactly_on_the_quadrant_angles() {
        let (sine, cose) = (23.44_f64.to_radians().sin(), 23.44_f64.to_radians().cos());
        assert_eq!(
            asc1(180.0, 0.0, sine, cose),
            180.0,
            "第三象限在 x=180 处给出 180+1e-10，吸附后应恰好是 180"
        );
        assert_eq!(asc1(90.0, 0.0, sine, cose), 90.0);
        assert_eq!(asc1(270.0, 0.0, sine, cose), 270.0);
    }

    /// 极高恰在 ±90° 时，`asc1` 直接给出 180 或 0，不进象限分发。
    ///
    /// 这两支（`(90 − f).abs() < VERY_SMALL` 与 `(90 + f).abs() < …`）是 swehouse.c 的极点保护。
    /// 上面那张表走不到它们：表里 `f` 最大 66°，而这两支只在正好 ±90 时开。
    #[test]
    fn the_pole_guards_short_circuit_before_the_quadrants() {
        let (sine, cose) = (23.44_f64.to_radians().sin(), 23.44_f64.to_radians().cos());
        for &x in &[0.0_f64, 45.0, 123.4, 200.0, 359.9] {
            assert!(
                (asc1(x, 90.0, sine, cose) - 180.0).abs() < 1e-12,
                "f=90° 时 asc1({x}) 应恒为 180"
            );
            assert!(
                asc1(x, -90.0, sine, cose).abs() < 1e-12,
                "f=−90° 时 asc1({x}) 应恒为 0"
            );
        }
    }

    // —— Diana, Princess of Wales (Rodden AA): 1961-07-01 19:45 BST=UT 18:45 ——
    // Sandringham 52°50′N 0°30′E。
    // Astrodienst 给出的 Placidus 完整宫尖（转十进制度）：
    //   1=258.4167°（射手18°25'）  2=299.8167°（摩羯29°49'）  3=348.3667°（双鱼18°22'）
    //   5=46.0500°（金牛16°03'）   6=63.3000°（双子03°18'）   10=203.0500°（天秤23°03'）
    //   11=226.0667°（天蝎16°04'） 12=243.3000°（射手03°18'）
    // 4/7/8/9 由对宫等同性派生。geocult.org 给出的 11=226.0594° / 12=243.2997° 与上一致。
    //
    // 验证容差：0.05°（角分级，匹配本算用平恒星时/平交角、ELP 截断的精度）。
    /// 第二张实测盘，换一个纬度。
    ///
    /// 唯一的外部锚此前只有戴安娜一张——一个 RAMC、一个纬度（52°50′N）。宫尖是随
    /// RAMC 与纬度两个参数变的曲面，一点定不住它：变异测试在 `cusps` / `solve_cusp` /
    /// `asc1` 上留下的活口，多半只是「那一点上恰好没差别」。
    ///
    /// 爱因斯坦：1879-03-14 11:30 LMT，乌尔姆 48°24′N 10°00′E，纬度比戴安娜低四度半。
    /// 升点与中天两源相合：
    ///
    /// 1. <https://www.astro.com/astro-databank/Einstein,_Albert> 一系的排盘细目，
    ///    作升 11°39′ 巨蟹、中天 12°50′ 双鱼，并给出二宫 28°37′ 巨蟹、三宫 17°48′ 狮子、
    ///    五宫 18°20′ 天秤、六宫 3°06′ 射手
    /// 2. <https://www.astrotheme.com/astrology/Albert_Einstein> 作升 11°38′ 巨蟹、
    ///    中天 12°50′ 双鱼（升点两源差一角分）
    ///
    /// 六个值本实现逐分复现。中间四个宫尖只有第一源给出，这一点写明——不假装两源。
    ///
    /// 另有一家（astro-charts）把时区记作 UTC+0:53 而非乌尔姆的 LMT+0:40，故整盘差
    /// 三度余。按它自己的偏移重算，本实现给 8°56′ 而它写 8°43′，仍差十余角分，
    /// 多半是 ΔT 或取整约定不同——不构成干净的旁证，故不取。
    #[test]
    fn einstein_placidus_cusps_at_another_latitude() {
        // 乌尔姆 10°00′E 的地方平时 = 10/15 小时。
        let m = mingli_astro::Moment::new(1879, 3, 14, 11, 30, 10.0 / 15.0);
        let (lat, lon) = (48.4, 10.0);
        let ramc = (m.sidereal_time + lon).rem_euclid(360.0);
        let (asc, mc) = crate::asc_mc(ramc, m.obliquity, lat);
        let cs = cusps(ramc, m.obliquity, lat, asc, mc).expect("乌尔姆非极区");

        // (宫号, 公布度数)：巨蟹起 90°、狮子 120°、天秤 180°、射手 240°、双鱼 330°。
        let expected: [(usize, f64); 6] = [
            (1, 90.0 + 11.0 + 38.0 / 60.0),
            (2, 90.0 + 28.0 + 37.0 / 60.0),
            (3, 120.0 + 17.0 + 48.0 / 60.0),
            (5, 180.0 + 18.0 + 20.0 / 60.0),
            (6, 240.0 + 3.0 + 6.0 / 60.0),
            (10, 330.0 + 12.0 + 50.0 / 60.0),
        ];
        for (k, want) in expected {
            let got = cs.cusps[k];
            let diff = signed_diff_deg(got, want).abs();
            assert!(
                diff < 1.0 / 60.0,
                "第 {k} 宫：算出 {got:.4}°，公布 {want:.4}°，差 {:.2} 角分",
                diff * 60.0
            );
        }
        // 对宫等同性在这张盘上同样成立（与戴安娜那条是同一条规则的另一处取样）。
        for k in 1..=6usize {
            assert!(
                signed_diff_deg(cs.cusps[k + 6], cs.cusps[k] + 180.0).abs() < 1e-9,
                "第 {k} 宫与第 {} 宫应正对",
                k + 6
            );
        }
    }

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

    /// 十二宫尖随 RAMC 走一圈：各自连续，且始终按序前进。
    ///
    /// 唯一的外部锚（Diana 那张盘）钉的是一个 RAMC、一个纬度上的十二个数。迭代解算
    /// 里的算术改坏了，只要那一点仍在容差内，就没人察觉——变异测试在 `cusps` /
    /// `solve_cusp` / `pack` 上留了三十来个活口。
    ///
    /// 这里不引新出处，用它必须具备的形状：宫尖是天球上连续移动的点，且十二宫首尾
    /// 相接绕一圈，任意相邻两宫的前进量恒在 (0°,180°) 内。解算跑偏就会撕开或倒序。
    ///
    /// 实测（2026-08-25）：0.25° 的 RAMC 步长上，六个纬度最大跳变 0.27°–1.09°
    /// （高纬宫尖走得快），顺序违反 0 次。测试取 0.5° 步长、阈值 3°。
    #[test]
    fn the_twelve_cusps_walk_the_circle_in_order_without_a_jump() {
        const EPS: f64 = 23.44;
        const STEP: f64 = 0.5;
        let mut worst = 0.0f64;
        for lat in [0.0f64, 40.0, 52.833, -35.0, 60.0] {
            let mut prev: Option<[f64; 13]> = None;
            let mut ramc = 0.0;
            while ramc < 360.0 {
                let (asc, mc) = crate::asc_mc(ramc, EPS, lat);
                let cs = cusps(ramc, EPS, lat, asc, mc)
                    .unwrap_or_else(|| panic!("纬度 {lat}° 不在极区，应有宫尖"));
                // 首尾相接绕一圈：相邻两宫的前进量恒在 (0,180)。
                for k in 1..=12usize {
                    let next = if k == 12 { 1 } else { k + 1 };
                    let ahead = (cs.cusps[next] - cs.cusps[k]).rem_euclid(360.0);
                    assert!(
                        ahead > 0.0 && ahead < 180.0,
                        "纬度 {lat}° RAMC {ramc}°：{k} 宫 {:.4}° 到 {next} 宫 {:.4}° 前进 {ahead:.4}°，不成序",
                        cs.cusps[k],
                        cs.cusps[next]
                    );
                }
                if let Some(before) = prev {
                    for (k, (now, was)) in
                        cs.cusps.iter().zip(before.iter()).enumerate().skip(1)
                    {
                        let moved = ((now - was + 180.0).rem_euclid(360.0) - 180.0).abs();
                        assert!(
                            moved < 3.0,
                            "纬度 {lat}° RAMC {ramc}°：{k} 宫跳了 {moved:.4}°"
                        );
                        worst = worst.max(moved);
                    }
                }
                prev = Some(cs.cusps);
                ramc += STEP;
            }
        }
        assert!(worst < 2.5, "最大跳变 {worst:.4}° 已逼近阈值，形状变了");
    }

    /// 半弧三分的那两个初值，进了迭代就被忘掉。
    ///
    /// 变异测试在 `cusps` 与 `solve_cusp` 上留下大批活口，追下去才发现相当一部分
    /// 落在 `fh1` / `fh2` 上——它们只喂给 `solve_cusp` 算第一个 `first_lambda`，
    /// 循环随后从收敛中的 cusp 自行重算极高。把 `fh2` 里的 `a_aux * 2.0` 改成
    /// `a_aux / 2.0`，Diana 那张盘的十二个宫尖一个数都不动。
    ///
    /// 那不是缺口，是等价变异。这条测试把「不是缺口」变成一件被钉住的事：初值给得
    /// 再离谱，收敛到的宫尖也一样。日后谁把初值改成承重的，这里会红。
    #[test]
    fn the_iteration_forgets_the_pole_it_started_from() {
        let (sine, cose) = (23.44f64.to_radians().sin(), 23.44f64.to_radians().cos());
        let tanfi = 52.833f64.to_radians().tan();
        for rectasc in [30.0f64, 95.0, 187.5, 260.0, 349.0] {
            for denom in [1.5f64, 3.0] {
                let reference = solve_cusp(rectasc, 0.0, denom, tanfi, sine, cose);
                for seed in [-45.0f64, -5.0, 12.5, 37.0, 60.0] {
                    let got = solve_cusp(rectasc, seed, denom, tanfi, sine, cose);
                    match (reference, got) {
                        (Some(a), Some(b)) => assert!(
                            signed_diff_deg(a, b).abs() < 1e-6,
                            "赤经 {rectasc}° 分母 {denom}：初值 0° 收到 {a}°，初值 {seed}° 收到 {b}°"
                        ),
                        (None, None) => {}
                        _ => panic!("赤经 {rectasc}° 分母 {denom}：初值 {seed}° 与初值 0° 一个有解一个无解"),
                    }
                }
            }
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

    /// 迭代的收敛判据全靠它，而它此前没有一条自己的测试。
    ///
    /// 把整个函数换成常数 0 时，`solve_cusp` 的循环第一轮就认为已收敛，交出初值而非解，
    /// 宫尖最多偏 598.87″（近 10 角分）——公开盘只精确到角分，取样点又都落在初值与解接近
    /// 的位置，所以整套测试一条都不红。这里按定义直接钉住。
    #[test]
    fn signed_diff_deg_is_the_short_way_round_and_keeps_its_sign() {
        // 跨 0° 时走短弧，且方向不同号相反。
        assert!((signed_diff_deg(10.0, 350.0) - 20.0).abs() < 1e-12);
        assert!((signed_diff_deg(350.0, 10.0) + 20.0).abs() < 1e-12);
        // 半圈是闭端：值域 (-180， 180]，所以正对面取 +180 而不是 -180。
        assert!((signed_diff_deg(180.0, 0.0) - 180.0).abs() < 1e-12);
        assert!((signed_diff_deg(0.0, 180.0) - 180.0).abs() < 1e-12);
        assert!(signed_diff_deg(42.0, 42.0).abs() < 1e-12);
        // 绕多少圈都不改变答案，且与直接作差在短弧内一致。
        for a in (0..360).step_by(11) {
            for b in (0..360).step_by(7) {
                let (a, b) = (f64::from(a), f64::from(b));
                let d = signed_diff_deg(a, b);
                assert!(d > -180.0 && d <= 180.0, "signed_diff_deg({a}， {b}) = {d} 出界");
                assert!(
                    (signed_diff_deg(a + 720.0, b - 360.0) - d).abs() < 1e-9,
                    "整圈应无影响：{a}， {b}"
                );
                // 定义式：b 加上这个差就回到 a。
                assert!(
                    norm360(b + d) - norm360(a) < 1e-9 || (norm360(b + d) - norm360(a)).abs() > 359.9,
                    "b + diff 应回到 a：{a}， {b}， {d}"
                );
            }
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
        // 边界本身。上面那行加了 1e-6，正好跳过唯一有争议的那个点：宫尖属于它开启的那一宫，
        // 还是属于它结束的那一宫。区间取半开 [lo， hi)，所以答案是前者，而这要求区间判定
        // 用严格小于——写成 `<=` 时，落在 cusp k 上的黄经会被判给 k-1，全盘偏一宫。
        for k in 1..=12u8 {
            assert_eq!(
                house_of(&cs, cs.cusps[k as usize]),
                k,
                "黄经恰在 cusp {k} 上，应归入第 {k} 宫"
            );
        }
    }

    #[test]
    fn asc2_quadrant_sanity() {
        // 原先两条断言都是 `is_finite()`——几乎什么值都过，两条快路走到哪里都看不出来。
        // 改成钉住文档里写明的取值。
        let (sine, cose) = (0.3977, 0.9175);
        // `ass` 被吸附到 0：出口按 sin_x 的正负取 ±90，负的那支再加 180，故两侧都是 90。
        assert!((asc2(90.0, 0.0, sine, cose) - 90.0).abs() < 1e-9, "ass=0、sin_x>0");
        assert!((asc2(270.0, 0.0, sine, cose) - 90.0).abs() < 1e-9, "ass=0、sin_x<0");
        // `sin_x` 被吸附到 0：出口是 ±VERY_SMALL，负的那支加 180 落到 180 之下一丝。
        assert!((asc2(1e-12, 10.0, sine, cose) - 1e-10).abs() < 1e-15, "sin_x=0、ass>0");
        assert!((asc2(180.0, 10.0, sine, cose) - (180.0 - 1e-10)).abs() < 1e-9, "sin_x=0、ass<0");
        // 值域恒在 [0,180)。
        for x in (0..3600).map(|k| f64::from(k) / 10.0) {
            let v = asc2(x, 30.0, sine, cose);
            assert!((0.0..180.0).contains(&v), "asc2({x}) = {v} 越界");
        }
    }

    /// `asc1` 四个象限之间接得上不接得上。
    ///
    /// 它把 `x1` 按象限分发给 `asc2`，二、三象限还要翻转极高 `f` 的符号——
    /// 变异测试在这四支上留了二十来个活口，因为唯一的外部锚（Diana 那张盘）
    /// 只走到其中一两个象限。
    ///
    /// 这里不引新的出处，改用它必须具备的形状：黄道与「极高线」的交点随赤经**连续**
    /// 且**单调**地走一圈。象限接错就会在边界上撕开一道口子。
    ///
    /// 实测（2026-08-25）：干净时 0.05° 的步长上最大跳变 0.127°、六组极高全单调；
    /// 把第三象限的 `-f` 写成 `f`，跳变涨到 8°–55°，六组里四组单调性也破。
    /// 阈值取 1°，最弱的一组仍有八倍余量。
    #[test]
    fn the_four_quadrants_of_asc1_join_up_without_a_seam() {
        const STEP: f64 = 0.05;
        let (sine, cose) = (23.44f64.to_radians().sin(), 23.44f64.to_radians().cos());
        let mut worst = 0.0f64;
        for pole in [0.0f64, 10.0, -10.0, 30.0, -30.0, 45.0, 52.833, 60.0, -60.0] {
            let mut prev = asc1(0.0, pole, sine, cose);
            let mut x = STEP;
            while x < 360.0 {
                let cur = asc1(x, pole, sine, cose);
                let step = (cur - prev + 360.0).rem_euclid(360.0);
                assert!(
                    step < 180.0,
                    "极高 {pole}° 在 x={x}° 处倒退了：{prev}° → {cur}°"
                );
                assert!(
                    step < 1.0,
                    "极高 {pole}° 在 x={x}° 处跳了 {step}°——象限没接上"
                );
                worst = worst.max(step);
                prev = cur;
                x += STEP;
            }
        }
        assert!(worst < 0.5, "最大跳变 {worst}° 已逼近阈值，形状变了");
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

    /// 下半弧的三分，扫一圈而不是钉一个点。
    ///
    /// 上面两条只验 11/12，它们由 MC 直接派生；2/3 由 IC 派生，而 IC 是这个函数里唯一
    /// 一处独立算出来的量。它此前完全没有被检查：唯一碰到 2/3 的测试取 MC=180°，
    /// 而 180+180 与 180×180 在模 360 下同为 0——那一个取样点上，正确的 IC 与一个
    /// 彻底错误的 IC 恰好重合。换任何别的 MC 都不会重合，所以这里扫一圈。
    #[test]
    fn porphyry_lower_arc_trisects_from_the_ic() {
        let mut checked = 0;
        for mc in (0..360).step_by(7) {
            for delta in (10..350).step_by(23) {
                let mc = f64::from(mc);
                let asc = norm360(mc + f64::from(delta));
                let cs = porphyry_cusps(asc, mc);
                let ic = norm360(mc + 180.0);
                assert!(
                    signed_diff_deg(cs.cusps[4], ic).abs() < 1e-9,
                    "第 4 宫尖应正是 IC：MC={mc} Asc={asc}"
                );
                // IC→DC 弧与 MC→Asc 弧等长，2/3 在其上三分。
                let arc = norm360(asc - mc);
                for (k, part) in [(2usize, arc / 3.0), (3, 2.0 * arc / 3.0)] {
                    let want = norm360(ic + part);
                    assert!(
                        signed_diff_deg(cs.cusps[k], want).abs() < 1e-9,
                        "第 {k} 宫尖偏了：MC={mc} Asc={asc} 得 {} 期望 {want}",
                        cs.cusps[k]
                    );
                    checked += 1;
                }
            }
        }
        assert!(checked > 500, "只验了 {checked} 个宫尖，取样太少");
    }

    #[test]
    fn asc1_pole_attractors() {
        let s = 23.44_f64.to_radians().sin();
        let c = 23.44_f64.to_radians().cos();
        assert!((asc1(45.0, 89.999_999_999_999_9, s, c) - 180.0).abs() < 1e-6);
        assert!((asc1(45.0, -89.999_999_999_999_9, s, c) - 0.0).abs() < 1e-6);
    }
}
