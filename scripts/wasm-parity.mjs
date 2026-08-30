// 把 native 与 wasm 各排一遍的全树逐字段比。判在这里，量在两边各自的入口。
//
// 允许什么、不允许什么，说清楚：
//   · 非数值的字段（字符串、整数、结构、键序）必须完全一致，差一处即红；
//   · 数值字段允许极小的相对差，因为两个目标用的不是同一套三角函数——
//     wasm32-unknown-unknown 没有系统 libm，模块把 compiler-builtins 那套编了进去
//     （其导入表只有两项 wasm-bindgen 胶水，无任何宿主数学函数），native 链的是系统 libm。
//   · 上限取 1e-10 相对差。360 度的 1e-10 是 0.13 毫角秒，远在 VSOP87D 自身截断误差
//     （约 1 角秒）之下；实测 2026-08-25 最大 2.6e-14，落在 37 个字段上，全在星历三叶。
import { readFileSync } from 'node:fs';

const LIMIT = 1e-10;
const [nativePath, wasmPath] = process.argv.slice(2);
const lines = (p) => readFileSync(p, 'utf8').split('\n').filter((l) => l.trim());
const nativeLines = lines(nativePath);
const wasmLines = lines(wasmPath);

// 首行是输入清单。两边的时刻表各写在各自的源文件里，先核对它们确是同一组，
// 否则后面比的是两组不同输入，比出「一致」也没有意义。
if (nativeLines[0] !== wasmLines[0]) {
  console.error('两边的输入清单不是同一组——比下去没有意义：');
  console.error('  native  ' + nativeLines[0]);
  console.error('  wasm    ' + wasmLines[0]);
  process.exit(1);
}
console.log('  输入清单两边一致：' + nativeLines[0]);

const a = nativeLines.slice(1).map((l) => JSON.parse(l));
const b = wasmLines.slice(1).map((l) => JSON.parse(l));

const structural = [];
const numeric = [];
let numericTotal = 0;

function walk(x, y, path) {
  if (typeof x !== typeof y || Array.isArray(x) !== Array.isArray(y)) {
    structural.push(path + ': 类型不同');
  } else if (Array.isArray(x)) {
    if (x.length !== y.length) {
      structural.push(path + ': 长度 ' + x.length + ' vs ' + y.length);
      return;
    }
    x.forEach((v, i) => walk(v, y[i], path + '[' + i + ']'));
  } else if (x !== null && typeof x === 'object') {
    if (Object.keys(x).join(' ') !== Object.keys(y).join(' ')) {
      structural.push(path + ': 键或键序不同');
      return;
    }
    for (const k of Object.keys(x)) walk(x[k], y[k], path + '.' + k);
  } else if (typeof x === 'number') {
    // JSON 不分整数与浮点。两边都是整数的（卦序、爻值、宫位、种子这类）要求严格相等——
    // 否则一个大整数差 1 会被相对容差吃掉。只要有一边不是整数，才走浮点那条路；
    // 这样 60.0 与 59.99999999999999 这种纯末位噪声不会被误判。
    if (Number.isInteger(x) && Number.isInteger(y)) {
      if (x !== y) structural.push(path + ': 整数 ' + x + ' vs ' + y);
      return;
    }
    numericTotal += 1;
    if (x !== y) {
      const rel = Math.abs(x - y) / Math.max(Math.abs(x), Math.abs(y), Number.MIN_VALUE);
      numeric.push({ path, x, y, rel });
    }
  } else if (x !== y) {
    structural.push(path + ': ' + JSON.stringify(x) + ' vs ' + JSON.stringify(y));
  }
}

if (a.length !== b.length) {
  console.error('两边查询数不同：native ' + a.length + ' / wasm ' + b.length);
  process.exit(1);
}
a.forEach((x, i) => walk(x, b[i], '[' + i + ']'));

const worst = numeric.reduce((m, d) => (d.rel > m ? d.rel : m), 0);
console.log('  浮点字段 ' + numericTotal + ' 个，不同 ' + numeric.length + ' 个，最大相对差 ' + worst.toExponential(2));
console.log('  非数值差异 ' + structural.length + ' 处');

let bad = false;
if (structural.length > 0) {
  console.error('');
  console.error('非数值的内容对不上——这不是浮点的事：');
  for (const s of structural.slice(0, 10)) console.error('  ' + s);
  bad = true;
}
if (worst > LIMIT) {
  console.error('');
  console.error('数值差超出 ' + LIMIT.toExponential() + '：');
  for (const d of numeric.filter((d) => d.rel > LIMIT).slice(0, 10)) {
    console.error('  ' + d.path + '  native ' + d.x + '  wasm ' + d.y + '  相对差 ' + d.rel.toExponential(2));
  }
  bad = true;
}
if (bad) process.exit(1);
console.log('');
console.log('两扇门同一棵树：非数值逐字节相同，数值差在最后几位');
