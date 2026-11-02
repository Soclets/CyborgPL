// Builds a Lumen program to a native binary the way Lumen's own bench does
// (bench/vs-lang/run.mjs, buildLumenBinary): compileToIR, optimizeIR, the self-hosted C
// emitter native/emit_fn.lm, then clang -O3. Lumen's own repo is at $LUMEN_DIR.
//     node lumen_build.mjs <program.lm> <out>
import fs from 'node:fs';
import path from 'node:path';
import { execFileSync } from 'node:child_process';

const root = process.env.LUMEN_DIR;
if (!root) { console.error('set LUMEN_DIR to a checkout of lumen-source/lumen'); process.exit(2); }
const [lmPath, out] = process.argv.slice(2);
const { compileToIR, optimizeIR, emitWith, EMIT_FN_BASE, EMIT_FN_CEIL } = await import(path.join(root, 'native', 'pipeline.mjs'));

const ir = await compileToIR(fs.readFileSync(lmPath, 'utf8'));
const { words, main } = await optimizeIR(ir.words, ir.main);
// The string pointers the optimized code still uses, as Lumen's bench collects them.
const ptrs = [];
let pc = 0;
while (pc < words.length) {
  const op = words[pc];
  if (op === 57) { pc = pc + 3 + words[pc + 1]; continue; }
  if (op === 15) ptrs.push(words[pc + 1]);
  let oplen = 0;
  if ([1, 2, 6, 7, 13, 14, 15, 25].includes(op)) oplen = 1;
  else if ([8, 29, 64].includes(op)) oplen = 2;
  pc = pc + 1 + oplen;
}
const byPtr = new Map(ir.strings.map(s => [s.ptr, s]));
const strings = [...new Set(ptrs)].map(p => { const s = byPtr.get(p); if (!s) throw new Error(`missing string ptr ${p}`); return s; });
const emitter = fs.readFileSync(path.join(root, 'native', 'emit_fn.lm'), 'utf8');
const csrc = await emitWith(emitter, words, main, strings, EMIT_FN_BASE, EMIT_FN_CEIL);
fs.writeFileSync(out + '.c', csrc);
execFileSync('clang', ['-O3', '-o', out, out + '.c'], { stdio: ['ignore', 'ignore', 'inherit'] });
