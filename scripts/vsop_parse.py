"""解析 vsop87 crate 的 VSOP87D 常量表。

两个坑，都踩过：
  1. 系数里会写 `PI` 这个字面常量，不是数字；
  2. 单元素表写在一行 `= [[a], [b], [c]];`，没有换行，
     以 `\n];` 收尾的正则会一路吃进下一个声明，把后面的表也弄脏。
"""
import re, math, pathlib

TOKEN = re.compile(r'-?\d+\.\d+(?:e-?\d+)?|-?PI')
DECL = re.compile(r'const ([LBR])(\d): \[\[f64; (\d+)\]; 3\] = \[([\s\S]*?)\];')

def _num(t):
    return math.pi * (-1 if t.startswith('-') else 1) if t.endswith('PI') else float(t)

def load(root, planet):
    src = pathlib.Path(f"{root}/src/vsop87d/{planet}.rs").read_text()
    out, bad = {}, []
    for m in DECL.finditer(src):
        kind, order, n, body = m.group(1), int(m.group(2)), int(m.group(3)), m.group(4)
        rows = re.findall(r'\[([^\[\]]*?)\]', body)
        cols = [[_num(t) for t in TOKEN.findall(r)] for r in rows]
        if len(rows) != 3 or not all(len(c) == n for c in cols):
            bad.append(f"{planet}.{kind}{order}"); continue
        out.setdefault(kind, {})[order] = list(zip(*cols))
    return out, bad

def series(d, kind, tau, thr=0.0):
    return sum(sum(A * math.cos(B + C * tau) for (A, B, C) in terms if abs(A) >= thr) * tau ** o
               for o, terms in sorted(d.get(kind, {}).items()))
