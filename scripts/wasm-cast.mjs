// 在 node 里通过 wasm 那扇门排同一批盘。这支只负责量，判在 wasm-parity.mjs。
//
// 时刻表必须与 `crates/mingli-registry/examples/tree_fingerprint.rs` 里的一字不差——
// 两边比的若不是同一份输入，比出来的东西没有意义。
import { createRequire } from 'node:module';
import { writeFileSync } from 'node:fs';

const require = createRequire(import.meta.url);
const [modulePath, outPath] = process.argv.slice(2);
const wasm = require(modulePath);

const MOMENTS = [
  [1990, 6, 15, 14, 30],
  [1987, 9, 17, 12, 0],
  [2024, 2, 4, 0, 1],
  [2000, 1, 1, 23, 59],
  [2050, 12, 31, 6, 6],
  [1901, 3, 21, 18, 45],
];

// 首行写输入清单，格式与 Rust 那边的 `{MOMENTS:?}` 对齐，供比较器核对两边同源。
let blob = '[' + MOMENTS.map((m) => '(' + m.join(', ') + ')').join(', ') + ']\n';
for (const [year, month, day, hour, minute] of MOMENTS) {
  const query = { year, month, day, hour, minute, tz: 8.0, gender: 'Male', seed: 20260825 };
  blob += wasm.cast(JSON.stringify(query)) + '\n';
}
writeFileSync(outPath, blob);
console.log('  wasm 排完 ' + MOMENTS.length + ' 个时刻，' + Buffer.byteLength(blob, 'utf8') + ' 字节');
