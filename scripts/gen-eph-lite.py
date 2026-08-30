#!/usr/bin/env python3
"""按振幅截断 VSOP87D，生成 `mingli-ephemeris` 的轻量表。

    ./scripts/gen-eph-lite.py [阈值]        # 缺省 1e-6

系数取自 `vsop87` crate（MIT/Apache-2.0）的 VSOP87D 表，那份表自身来自
Bureau des Longitudes 的官方 VSOP87 参考文件。这里做的只是**丢掉振幅低于阈值的项**——
留下的是已被验证过的那份系数的子集，不是新写的权威值，所以残差对着全量量即可，
不需要第二个来源。

生成前会自检：不丢任何项时，本脚本的求值必须与 `vsop87` crate 的输出一致。
不一致就不生成——曾经两次解析出错（系数里写着 `PI` 字面量；单元素表写在一行上，
以换行收尾的正则会吃进下一个声明），而两次都是这一步拦下的。
"""
import sys, math, pathlib, subprocess, re
sys.path.insert(0, str(pathlib.Path(__file__).parent))
from vsop_parse import load, series

ROOT = pathlib.Path(__file__).resolve().parent.parent
PLANETS = ("mercury", "venus", "earth", "mars", "jupiter", "saturn", "uranus", "neptune")


def registry_src() -> str:
    """找到本机 cargo registry 里 vsop87 crate 的源码目录。"""
    base = pathlib.Path.home() / ".cargo/registry/src"
    hits = sorted(base.glob("*/vsop87-3.*"))
    if not hits:
        sys.exit("找不到 vsop87 crate 源码——先 `cargo fetch`")
    return str(hits[-1])


def emit(tables, thr, out_dir):
    out_dir.mkdir(parents=True, exist_ok=True)
    total = 0
    for planet in PLANETS:
        d = tables[planet]
        lines = [
            "//! 由 `scripts/gen-eph-lite.py` 生成，不要手改。",
            "//!",
            "//! VSOP87D 系数取自 `vsop87` crate（MIT/Apache-2.0），其表来自 Bureau des",
            f"//! Longitudes 的官方参考文件；此处只丢掉振幅低于 {thr:g} 的项。",
            "",
        ]
        # 六阶一律输出，源里没有的补空数组——调用方的宏按固定形状取，
        # 少一个就编不过，而「海王没有 R5」这种事各行星并不一致。
        for kind in ("L", "B", "R"):
            for order in range(6):
                terms = [t for t in d.get(kind, {}).get(order, []) if abs(t[0]) >= thr]
                total += len(terms)
                body = ",\n".join(f"    [{a!r}, {b!r}, {c!r}]" for a, b, c in terms)
                lines.append(
                    f"pub(crate) const {kind}{order}: [[f64; 3]; {len(terms)}] = ["
                    + ("\n" + body + ",\n" if terms else "")
                    + "];"
                )
        (out_dir / f"{planet}.rs").write_text("\n".join(lines) + "\n")
    return total


def main():
    thr = float(sys.argv[1]) if len(sys.argv) > 1 else 1e-6
    src = registry_src()
    tables = {}
    for p in PLANETS:
        t, bad = load(src, p)
        if bad:
            sys.exit(f"解析失败：{bad}")
        tables[p] = t

    # 自检：不丢项时必须与参考实现一致。
    print("自检：全量求值对照 vsop87 crate")
    ref = subprocess.run(
        ["cargo", "run", "-q", "--release", "--example", "vsop_reference"],
        cwd=ROOT, capture_output=True, text=True,
    )
    if ref.returncode != 0:
        sys.exit("参考值取不到：\n" + ref.stderr[-800:])
    worst = 0.0
    n = 0
    for line in ref.stdout.splitlines():
        p, jde, L, B, R = line.split()
        tau = (float(jde) - 2451545.0) / 365250.0
        for kind, want in (("L", float(L)), ("B", float(B)), ("R", float(R))):
            got = series(tables[p], kind, tau)
            if kind == "L":
                got %= 2 * math.pi
            worst = max(worst, abs(got - want))
            n += 1
    if n < 100:
        sys.exit(f"只对照了 {n} 个值，参考输出怕是不对")
    if worst > 1e-9:
        sys.exit(f"全量求值与参考实现差 {worst:.3e}，不生成")
    print(f"  ✓ {n} 个值，最大差 {worst:.3e}")

    kept = emit(tables, thr, ROOT / "crates/mingli-ephemeris/src/lite")
    full = sum(len(v) for p in PLANETS for k in tables[p] for v in tables[p][k].values())
    print(f"阈值 {thr:g}：{full} 项留 {kept}（{kept / full * 100:.1f}%）")


if __name__ == "__main__":
    main()
