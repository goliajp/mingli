//! 跨叶分析用例：固定采样网格 → NMI 矩阵。网格固定所以结果确定，首次算完即缓存。

use mingli_contract::CastingEngine;
use serde_json::Value;
use std::sync::OnceLock;

/// 采样网格的年份区间（固定 → 结果可复现）。
const GRID: (i32, i32) = (1980, 2009);

static CACHE: OnceLock<Value> = OnceLock::new();

/// 跨叶相关性分析（带进程内缓存）。
#[must_use]
pub fn cross_leaf_cached(reg: &[Box<dyn CastingEngine>]) -> Value {
    CACHE
        .get_or_init(|| {
            let a = mingli_analysis::cross_leaf(reg, &mingli_analysis::sample_grid(GRID.0, GRID.1));
            serde_json::to_value(a).unwrap_or(Value::Null)
        })
        .clone()
}
