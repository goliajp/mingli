//! L2 主干：洛书幻方与九宫飞布。
//!
//! 洛书是三阶幻方（每行/列/对角和恒 15）。九宫将 1..9 配后天八卦与方位。
//! 「飞布」是某数入中后沿洛书轨迹填布九宫的过程，本质是 `Z₉` 上的 ±k 群作用，
//! 由 [`mingli_core::group::flying_star`] 提供；本 crate 加上九宫的空间布局与领域语义。
//! 被奇门遁甲、紫白飞星、太乙等复用。

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    reason = "九宫数 1..9、路径下标 0..9：与 i64(core::group) 间的换算恒在受控小范围内"
)]

use mingli_core::group::flying_star;

/// 洛书三阶幻方（上南下北，戴九履一、左三右七、二四为肩、六八为足、五居中）。
/// `LUOSHU_GRID[row][col]`，row=0 为上（南）。
pub const LUOSHU_GRID: [[u8; 3]; 3] = [[4, 9, 2], [3, 5, 7], [8, 1, 6]];

/// 九宫名（后天八卦），按宫数 1..9 索引（index 0 占位）。
pub const PALACE_NAME: [&str; 10] = [
    "", "坎", "坤", "震", "巽", "中", "乾", "兑", "艮", "离",
];
/// 九宫方位，按宫数 1..9 索引（index 0 占位）。
pub const PALACE_DIR: [&str; 10] = [
    "", "北", "西南", "东", "东南", "中", "西北", "西", "东北", "南",
];

/// 飞星轨迹：从中宫起，依次经过的九宫「本位数」顺序（中5→乾6→兑7→艮8→离9→坎1→坤2→震3→巽4）。
pub const FLIGHT_PATH: [u8; 9] = [5, 6, 7, 8, 9, 1, 2, 3, 4];

/// 某数 `center` 入中飞布：返回各宫所得之数，按**本位数**索引（`out[n-1]` = 本位 n 宫所飞入之数）。
/// `forward=true` 阳顺（数递增）、`false` 阴逆（数递减）。
#[must_use]
pub fn fly(center: u8, forward: bool) -> [u8; 9] {
    let mut out = [0u8; 9];
    for (k, &native) in FLIGHT_PATH.iter().enumerate() {
        out[(native - 1) as usize] = flying_star(i64::from(center), k as i64, forward) as u8;
    }
    out
}

/// 本位数 `n`(1..9) 在三阶幻方中的 （行， 列）（行 0 为上/南）。
#[must_use]
pub fn grid_position(n: u8) -> (usize, usize) {
    for (r, row) in LUOSHU_GRID.iter().enumerate() {
        for (c, &v) in row.iter().enumerate() {
            if v == n {
                return (r, c);
            }
        }
    }
    (1, 1) // 不可达（n 越界时退回中宫）
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn magic_square_sums_15() {
        for (i, row) in LUOSHU_GRID.iter().enumerate() {
            assert_eq!(row.iter().sum::<u8>(), 15); // 行
            assert_eq!(LUOSHU_GRID.iter().map(|r| r[i]).sum::<u8>(), 15); // 列
        }
        assert_eq!((0..3).map(|i| LUOSHU_GRID[i][i]).sum::<u8>(), 15); // 主对角
        assert_eq!((0..3).map(|i| LUOSHU_GRID[i][2 - i]).sum::<u8>(), 15); // 副对角
        // 1..9 恰好各一次
        let mut seen = [false; 10];
        for row in LUOSHU_GRID {
            for v in row {
                seen[v as usize] = true;
            }
        }
        assert!(seen[1..=9].iter().all(|&b| b));
    }

    #[test]
    fn fly_center_5_is_natural() {
        // 五入中顺飞 = 本位盘：每宫飞入其本位数。
        assert_eq!(fly(5, true), [1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }

    #[test]
    fn fly_center_8_forward() {
        // 八运（8 入中顺飞）：中宫=8；乾6宫飞入 9；兑7宫飞入 1。
        let f = fly(8, true);
        assert_eq!(f[4], 8); // 中宫（本位5）
        assert_eq!(f[5], 9); // 乾宫（本位6）
        assert_eq!(f[6], 1); // 兑宫（本位7）
    }

    #[test]
    fn fly_reverse() {
        // 五入中逆飞 = 本位盘的镜像（中点 5 不动，余数递减）。
        assert_eq!(fly(5, false), [9, 8, 7, 6, 5, 4, 3, 2, 1]);
        let r = fly(6, false);
        assert_eq!(r[4], 6); // 中宫
        assert_eq!(r[5], 5); // 乾宫（本位6） 逆飞得 5
    }

    /// 飞布是九宫上的一个置换——穷举九个入中数 × 顺逆两向共十八盘。
    ///
    /// 在此之前只抽查了 5 / 6 / 8 入中的三两格。抽查看不出「某一格重复、另一格缺失」
    /// 这种最典型的模运算错法：错一处，被查的那几格照样对。
    /// 置换性是这一层唯一必须成立的东西——九宫各得一数、不重不漏，
    /// 上面所有术数的飞星盘都建立在它之上。
    #[test]
    fn flying_is_a_permutation_for_every_center_and_direction() {
        for center in 1..=9u8 {
            for forward in [true, false] {
                let f = fly(center, forward);
                let mut seen = [0u8; 10];
                for &v in &f {
                    assert!((1..=9).contains(&v), "{center} 入中{}飞得到 {v}，越出 1..=9",
                        if forward { "顺" } else { "逆" });
                    seen[v as usize] += 1;
                }
                assert!(
                    seen[1..=9].iter().all(|&c| c == 1),
                    "{center} 入中{}飞不是置换：{f:?}（各数出现次数 {:?}）",
                    if forward { "顺" } else { "逆" },
                    &seen[1..=9],
                );
                // 入中之数必落中宫（本位 5 那一格）
                assert_eq!(
                    f[4], center,
                    "{center} 入中{}飞，中宫应得 {center}，实得 {}",
                    if forward { "顺" } else { "逆" }, f[4],
                );
            }
        }
    }

    /// 顺飞与逆飞互为镜像：同一入中数下，两者在每一宫的取值关于中宫之数对称。
    ///
    /// 顺飞第 k 步得 `center + k`，逆飞得 `center − k`（皆模 9 归 1..9），
    /// 于是两数之和恒 ≡ 2×center（mod 9）。这条把「顺逆共用同一个群作用、
    /// 只差一个符号」钉住——某天有人把逆飞另写一套实现时，它会红。
    #[test]
    fn forward_and_reverse_are_mirror_images() {
        for center in 1..=9u8 {
            let (f, r) = (fly(center, true), fly(center, false));
            for p in 0..9 {
                let sum = i32::from(f[p]) + i32::from(r[p]);
                let want = 2 * i32::from(center);
                assert_eq!(
                    sum.rem_euclid(9), want.rem_euclid(9),
                    "{center} 入中：本位 {} 宫顺飞 {} 逆飞 {}，和模 9 应为 {}",
                    p + 1, f[p], r[p], want.rem_euclid(9),
                );
            }
        }
    }

    #[test]
    fn positions() {
        assert_eq!(grid_position(5), (1, 1)); // 中
        assert_eq!(grid_position(9), (0, 1)); // 南（上中）
        assert_eq!(grid_position(1), (2, 1)); // 北（下中）
        assert_eq!(grid_position(0), (1, 1)); // 越界 → 退回中宫
        assert_eq!(PALACE_NAME[9], "离");
        assert_eq!(PALACE_DIR[1], "北");
    }
}
