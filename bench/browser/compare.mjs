// 浏览器形态的对照：同一批输入、同一台机器，跟当地最强的 JS 实现比。
//
// 两件事分开做，且顺序不可换：
//   1. 先对拍。答案不同就不谈快慢——那是在比不同的活。
//   2. 再计时。交错跑、取中位数，免得先跑的那个吃冷启动。
//
// 四柱那一路 lunar-javascript 与我们逐字相同，所以它同时是对手和第二个来源。
// 星历那一路不是：我们与 astronomy-engine 的黄经差在几十角秒量级，而在它自己
// 随包发布的文档里查不到行星位置精度的说法，所以说不清那几十角秒是谁的。
// 正确性的判据仍是我们对 JPL / IMCCE 的 oracle 测试；这里只把差值记下来当漂移探测：
// 记录值在 eph-drift.txt，涨了就红，但它不是精度声明。
import { readFileSync, writeFileSync, existsSync } from 'node:fs';
import { createRequire } from 'node:module';
const require = createRequire(import.meta.url);
const { Solar } = require('lunar-javascript');
const A = require('astronomy-engine');

const root = new URL('../../dist/npm/', import.meta.url);
async function load(pkg) {
  const m = await import(new URL(`${pkg}/mingli_wasm.js`, root));
  await m.default({ module_or_path: readFileSync(new URL(`${pkg}/mingli_wasm_bg.wasm`, root)) });
  return m;
}
const med = (a) => a.slice().sort((x, y) => x - y)[a.length >> 1];
const stdev = (a) => { const m = a.reduce((s, x) => s + x, 0) / a.length;
  return Math.sqrt(a.reduce((s, x) => s + (x - m) ** 2, 0) / a.length); };
const time = (f, qs) => { const t = process.hrtime.bigint(); for (const q of qs) f(q); return Number(process.hrtime.bigint() - t) / 1e6; };
function race(name, mine, theirs, qs, rounds = 9) {
  for (let w = 0; w < 3; w++) { time(mine, qs); time(theirs, qs); }
  const A2 = [], B = [];
  for (let r = 0; r < rounds; r++) { A2.push(time(mine, qs)); B.push(time(theirs, qs)); }
  const a = med(A2), b = med(B);
  const rel = (xs) => (stdev(xs) / med(xs) * 100).toFixed(1);
  console.log(`  我们      ${(a * 1000 / qs.length).toFixed(2).padStart(8)} µs/次   方差 ${rel(A2)}%`);
  console.log(`  ${name.padEnd(9)}${(b * 1000 / qs.length).toFixed(2).padStart(8)} µs/次   方差 ${rel(B)}%`);
  console.log(`  比值      ${(b / a).toFixed(2)}× ${b > a ? '（我们快）' : '（我们慢）'}`);
  return b / a;
}

let seed = 20260830 >>> 0;
const rnd = () => (seed = (seed * 1103515245 + 12345) >>> 0) / 4294967296;
const moments = (n) => Array.from({ length: n }, () => {
  const year = 1900 + Math.floor(rnd() * 200), month = 1 + Math.floor(rnd() * 12);
  const dim = new Date(Date.UTC(year, month, 0)).getUTCDate();
  return { year, month, day: 1 + Math.floor(rnd() * dim), hour: Math.floor(rnd() * 24),
           minute: Math.floor(rnd() * 60), tz: 8.0, latitude: 31.23, longitude: 121.47 };
});

let fail = 0;
const N = Number(process.argv[2] ?? 2000);
const qs = moments(N);

// ---- 四柱 ----
console.log(`四柱 · 对手 lunar-javascript · ${N} 个时刻\n`);
const bz = await load('mingli-wasm-bazi');
const ours = (q) => { const c = JSON.parse(bz.cast_one('bazi', JSON.stringify(q))).chart;
  return c.year.ganzhi + c.month.ganzhi + c.day.ganzhi + c.hour.ganzhi; };
const lun = (q) => { const l = Solar.fromYmdHms(q.year, q.month, q.day, q.hour, q.minute, 0).getLunar();
  return l.getYearInGanZhiExact() + l.getMonthInGanZhiExact() + l.getDayInGanZhiExact() + l.getTimeInGanZhi(); };
let bad = 0, first = null;
for (const q of qs) if (ours(q) !== lun(q)) { bad++; first ??= q; }
if (bad) { console.log(`  ✗ 对拍分歧 ${bad}/${N}，首例 ${JSON.stringify(first)}`); fail = 1; }
else {
  console.log(`  ✓ 对拍 ${N} 个时刻逐字相同`);
  const r = race('lunar-js', ours, lun, qs);
  if (r < 1.0) { console.log('  ✗ 比对手慢'); fail = 1; }
}

// ---- 星历 ----
console.log(`\n星历 · 对手 astronomy-engine · 每十年一个取样`);
const ch = await load('mingli-wasm-chart');
const BODIES = [['太阳','Sun'],['月亮','Moon'],['水星','Mercury'],['金星','Venus'],['火星','Mars'],['木星','Jupiter'],['土星','Saturn']];
const wrap = (d) => { d = ((d % 360) + 360) % 360; return d > 180 ? d - 360 : d; };
const worst = new Map(BODIES.map(([zh]) => [zh, 0]));
for (let y = 1900; y <= 2100; y += 10) {
  const c = JSON.parse(ch.cast_one('astrology', JSON.stringify(
    { year: y, month: 6, day: 15, hour: 12, minute: 0, tz: 0.0, latitude: 0, longitude: 0 }))).chart;
  const d = new Date(Date.UTC(y, 5, 15, 12, 0, 0));
  for (const [zh, en] of BODIES) {
    const mine = c.planets.find((p) => p.name === zh)?.longitude;
    if (mine == null) { console.log(`  ✗ 我们没给出 ${zh}`); fail = 1; continue; }
    const diff = Math.abs(wrap(mine - A.Ecliptic(A.GeoVector(en, d, true)).elon)) * 3600;
    if (diff > worst.get(zh)) worst.set(zh, diff);
  }
}
const drift = new URL('eph-drift.txt', import.meta.url);
if (process.argv.includes('--record')) {
  writeFileSync(drift, '# 与 astronomy-engine 的黄经差上界（角秒）。不是精度声明——\n' +
    '# 对手自己的文档里查不到行星位置精度，说不清这几十角秒是谁的。\n' +
    '# 正确性的判据是我们对 JPL / IMCCE 的 oracle 测试；这张表只用来发现漂移。\n' +
    [...worst].map(([k, v]) => `${k} ${v.toFixed(1)}`).join('\n') + '\n');
  console.log('  已记录 eph-drift.txt');
} else if (existsSync(drift)) {
  for (const line of readFileSync(drift, 'utf8').split('\n')) {
    if (!line || line.startsWith('#')) continue;
    const [k, v] = line.split(' ');
    const got = worst.get(k);
    if (got == null) { console.log(`  ✗ 记录里有 ${k}，这次没量到`); fail = 1; }
    else if (got > Number(v) + 1.0) { console.log(`  ✗ ${k} 差 ${got.toFixed(1)}″，记录是 ${v}″——漂了`); fail = 1; }
  }
  if (!fail) console.log(`  ✓ 七个天体与记录相符（最大 ${Math.max(...worst.values()).toFixed(1)}″）`);
}
// 同口径：两边都只出黄经。曾经这里比的是「我们的整张盘」对「对手的七个黄经」，
// 我把差别归给了「不是同一件活」——消融量下来那是错的：整盘 286.7 µs、
// 只算位置 278.1 µs，星历占九成七，相位宫位落座合计才 9 µs。
// 现在两边都走位置那一条，剩下的差就真的是星历本身的差。
const ephOurs = (q) => ch.longitudes(JSON.stringify(q));
const ephTheirs = (q) => { const d = new Date(Date.UTC(q.year, q.month - 1, q.day, q.hour, q.minute, 0));
  for (const [, en] of BODIES) A.Ecliptic(A.GeoVector(en, d, true)); };
console.log('\n  （同口径：两边都只出黄经。我们九星、对手七星，仍不设闸，但差别只剩这一处）');
race('astro-eng', ephOurs, ephTheirs, qs.slice(0, 300), 5);

process.exit(fail);
