// The model behind the hero: a real simulation, stepped one millisecond of model time at a time.
//
// Neurons are Izhikevich's (2003) spiking model, 4 in 5 excitatory and the rest inhibitory, each
// driven by its own noise. Each grows axons: a growth cone moves forward, keeps most of its heading,
// is drawn up the gradient of an attractant every other cell body gives off, wanders a little, and
// now and then splits. When a cone reaches a cell body it makes a synapse; one that wanders off or
// grows too long pulls back. A spike then travels each axon at a fixed speed and lands on its target
// after the delay that length gives it.
//
// Once growth ends, the circuit takes over: each connection hardens into a trace on a grid, at 0, 45
// and 90 degrees, as spikes cross it, so the busiest pathways turn first, and a slow pressure turns
// the rest. A cell body hardens into a pad as its connections do. The moment every part has turned,
// the board is formed.

export type Pt = { x: number; y: number };

export const GRID = 16;           // the board's pitch, px
const K = 32;                     // points each path is resampled to, so the two shapes can blend

const BIRTH_SPAN = 250;           // ms over which the neurons appear
const CONE_SPEED = 0.45;          // px per ms
const PERSIST = 1;                // how much a cone keeps its heading…
const PULL = 1.4;                 // …against the attractant's pull
const WANDER = 0.3;               // the random turn, in radians, per ms
const LAMBDA = 120;               // px over which the attractant fades
const MAX_AXON = 340;             // px before a cone gives up
const BRANCH_P = 0.003;           // per ms
const ARRIVE = 6;                 // px from a cell body to synapse
const RETRACT = 1.4;              // px per ms a failed cone pulls back
const CONDUCTION = 2;             // px per ms a spike travels

export const CONVERT_AT = 900;    // ms: growth stops, the circuit takes over
const CONVERT_SPAN = 2600;        // ms the pressure takes to turn everything
const USE_FULL = 8;               // spikes that turn one connection by themselves
const MORPH_RATE = 0.004;         // per ms, how fast a connection follows its target
const FORMED = 0.97;              // how far every part must have turned for the board to count as formed
const HOLD = 1500;                // ms the formed board stays…
export const FADE = 150;          // …the last of which it fades over, before the next grows

export type Neuron = {
  x: number; y: number;           // the cell body
  px: number; py: number;         // its pad on the board
  bornAt: number;
  inhibitory: boolean;
  a: number; b: number; c: number; d: number;
  v: number; u: number;
  input: number;
  flash: number;                  // 1 at a spike, then decays
  hard: number;                   // 0 a cell body, 1 a pad
  degree: number;
};

export type Cone = { from: number; x: number; y: number; dx: number; dy: number; path: Pt[]; len: number; split: boolean; dying: boolean };

export type Edge = { from: number; to: number; w: number; delay: number; organic: Pt[]; trace: Pt[]; use: number; morph: number };

export type Pulse = { edge: Edge; at: number };

export class Sim {
  t = 0;
  readonly neurons: Neuron[] = [];
  cones: Cone[] = [];
  readonly edges: Edge[] = [];
  pulses: Pulse[] = [];
  formedAt: number | null = null;
  private readonly rand: () => number;
  private readonly linked = new Set<number>();

  readonly w: number;
  readonly h: number;

  constructor(w: number, h: number, seed: number) {
    this.w = w; this.h = h;
    this.rand = mulberry32(seed);
    const n = Math.round(Math.min(90, Math.max(16, (w * h) / 22000)));
    const gap = Math.sqrt((w * h) / n) * 0.62;
    const margin = 24;
    for (let tries = 0; this.neurons.length < n && tries < n * 60; tries++) {
      const x = margin + this.rand() * (w - 2 * margin);
      const y = margin + this.rand() * (h - 2 * margin);
      const px = Math.round(x / GRID) * GRID, py = Math.round(y / GRID) * GRID;
      if (this.neurons.some((o) => Math.hypot(o.x - x, o.y - y) < gap || (o.px === px && o.py === py))) continue;
      const inhibitory = this.rand() < 0.2;
      const r = this.rand();
      // Izhikevich's parameters: regular spiking spread towards chattering, fast spiking for the inhibitory.
      const p = inhibitory ? { a: 0.02 + 0.08 * r, b: 0.25 - 0.05 * r, c: -65, d: 2 } : { a: 0.02, b: 0.2, c: -65 + 15 * r * r, d: 8 - 6 * r * r };
      this.neurons.push({ x, y, px, py, bornAt: this.rand() * BIRTH_SPAN, inhibitory, ...p, v: -65, u: p.b * -65, input: 0, flash: 0, hard: 0, degree: 0 });
    }
  }

  get end() { return this.formedAt === null ? Infinity : this.formedAt + HOLD; }
  get done() { return this.t >= this.end; }

  /** Runs the model forward to `t` ms at once (for a still picture). */
  runTo(t: number) { while (this.t < t) this.step(); }

  step() {
    const t = ++this.t;
    if (t < CONVERT_AT) this.grow(t);
    else if (this.cones.some((c) => !c.dying)) for (const c of this.cones) c.dying = true;
    this.retract();

    // Spikes in flight land on their targets.
    const flying: Pulse[] = [];
    for (const p of this.pulses) {
      if (t - p.at < p.edge.delay) { flying.push(p); continue; }
      this.neurons[p.edge.to]!.input += p.edge.w;
      if (t >= CONVERT_AT) p.edge.use++;
    }
    this.pulses = flying;

    // The neurons: two half steps for v, as Izhikevich does, then u.
    for (let i = 0; i < this.neurons.length; i++) {
      const n = this.neurons[i]!;
      if (t < n.bornAt) continue;
      const I = n.input + (n.inhibitory ? 2 : 5) * this.gauss();
      n.input = 0;
      n.v += 0.5 * (0.04 * n.v * n.v + 5 * n.v + 140 - n.u + I);
      n.v += 0.5 * (0.04 * n.v * n.v + 5 * n.v + 140 - n.u + I);
      n.u += n.a * (n.b * n.v - n.u);
      n.flash *= 0.985;
      if (n.v >= 30) {
        n.v = n.c; n.u += n.d; n.flash = 1;
        for (const e of this.edges) if (e.from === i) this.pulses.push({ edge: e, at: t });
      }
    }

    // The circuit: each connection follows its spikes and the pressure; each cell body its connections.
    if (t >= CONVERT_AT) {
      const pressure = Math.min(1, (t - CONVERT_AT) / CONVERT_SPAN);
      const sum = new Float64Array(this.neurons.length);
      for (const e of this.edges) {
        const target = Math.min(1, Math.max(pressure, e.use / USE_FULL));
        e.morph += (target - e.morph) * MORPH_RATE;
        sum[e.from]! += e.morph; sum[e.to]! += e.morph;
      }
      this.neurons.forEach((n, i) => {
        const target = n.degree ? sum[i]! / n.degree : pressure;
        n.hard += (target - n.hard) * MORPH_RATE;
      });
      if (this.formedAt === null && this.edges.every((e) => e.morph > FORMED) && this.neurons.every((n) => n.hard > FORMED)) this.formedAt = t;
    }
  }

  private grow(t: number) {
    // New neurons send out their first three cones.
    this.neurons.forEach((n, i) => {
      if (t !== Math.ceil(n.bornAt) && !(n.bornAt < 1 && t === 1)) return;
      for (let k = 0; k < 3; k++) {
        const a = this.rand() * Math.PI * 2;
        this.cones.push({ from: i, x: n.x, y: n.y, dx: Math.cos(a), dy: Math.sin(a), path: [{ x: n.x, y: n.y }], len: 0, split: false, dying: false });
      }
    });

    const born: Cone[] = [];
    for (const c of this.cones) {
      if (c.dying) continue;
      // The attractant's gradient: every other cell body not yet reached from this neuron.
      let gx = 0, gy = 0;
      this.neurons.forEach((n, k) => {
        if (k === c.from || t < n.bornAt || this.linked.has(c.from * 4096 + k)) return;
        const dx = n.x - c.x, dy = n.y - c.y, d = Math.hypot(dx, dy);
        if (d < 1) return;
        const s = Math.exp(-d / LAMBDA) / d;
        gx += dx * s; gy += dy * s;
      });
      const g = Math.hypot(gx, gy) || 1;
      let dx = PERSIST * c.dx + PULL * (gx / g), dy = PERSIST * c.dy + PULL * (gy / g);
      const turn = WANDER * this.gauss(), cos = Math.cos(turn), sin = Math.sin(turn);
      [dx, dy] = [dx * cos - dy * sin, dx * sin + dy * cos];
      const m = Math.hypot(dx, dy) || 1;
      c.dx = dx / m; c.dy = dy / m;
      c.x += c.dx * CONE_SPEED; c.y += c.dy * CONE_SPEED; c.len += CONE_SPEED;
      const tip = c.path[c.path.length - 1]!;
      if (Math.hypot(c.x - tip.x, c.y - tip.y) >= 3) c.path.push({ x: c.x, y: c.y });

      const hit = this.neurons.findIndex((n, k) => k !== c.from && t >= n.bornAt && Math.hypot(n.x - c.x, n.y - c.y) < ARRIVE);
      if (hit >= 0) { this.synapse(c, hit); c.path = []; c.dying = true; continue; }
      if (c.len > MAX_AXON || c.x < 0 || c.y < 0 || c.x > this.w || c.y > this.h) { c.dying = true; continue; }
      if (!c.split && c.len > 30 && this.rand() < BRANCH_P) {
        c.split = true;
        const a = (this.rand() < 0.5 ? -1 : 1) * 0.7, s = Math.sin(a), k = Math.cos(a);
        born.push({ ...c, dx: c.dx * k - c.dy * s, dy: c.dx * s + c.dy * k, path: c.path.map((p) => ({ ...p })) });
      }
    }
    this.cones.push(...born);
  }

  private retract() {
    for (const c of this.cones) {
      if (!c.dying) continue;
      for (let left = RETRACT; left > 0 && c.path.length > 1; ) {
        const a = c.path[c.path.length - 1]!, b = c.path[c.path.length - 2]!;
        const d = Math.hypot(a.x - b.x, a.y - b.y);
        if (d <= left) { c.path.pop(); left -= d; } else { a.x += ((b.x - a.x) * left) / d; a.y += ((b.y - a.y) * left) / d; left = 0; }
      }
    }
    this.cones = this.cones.filter((c) => !c.dying || c.path.length > 1);
  }

  private synapse(c: Cone, to: number) {
    const key = c.from * 4096 + to;
    if (this.linked.has(key)) return;
    this.linked.add(key);
    const a = this.neurons[c.from]!, b = this.neurons[to]!;
    const path = [...c.path, { x: b.x, y: b.y }];
    const r = this.rand();
    this.edges.push({
      from: c.from, to,
      w: a.inhibitory ? -(10 + 8 * r) : 9 + 6 * r,
      delay: Math.max(4, Math.round(length(path) / CONDUCTION)),
      organic: resample(path, K),
      trace: resample(route(a, b, (c.from + to) % 2 === 0), K),
      use: 0, morph: 0,
    });
    a.degree++; b.degree++;
  }

  private spare: number | null = null;
  private gauss() {
    if (this.spare !== null) { const s = this.spare; this.spare = null; return s; }
    const u = this.rand() || 1e-9, v = this.rand(), r = Math.sqrt(-2 * Math.log(u));
    this.spare = r * Math.sin(2 * Math.PI * v);
    return r * Math.cos(2 * Math.PI * v);
  }
}

/** A board trace from pad to pad: straight, a 45° run, straight again (or the 45° first). */
function route(a: Neuron, b: Neuron, diagonalFirst: boolean): Pt[] {
  const dx = b.px - a.px, dy = b.py - a.py, sx = Math.sign(dx), sy = Math.sign(dy);
  const adx = Math.abs(dx), ady = Math.abs(dy), diag = Math.min(adx, ady);
  const flat = Math.round((Math.max(adx, ady) - diag) / 2 / GRID) * GRID;
  const lead = diagonalFirst ? 0 : flat;
  const along = (s: number) => (adx >= ady ? { x: sx * s, y: 0 } : { x: 0, y: sy * s });
  const p1 = along(lead), p2 = { x: p1.x + sx * diag, y: p1.y + sy * diag };
  return [{ x: a.px, y: a.py }, { x: a.px + p1.x, y: a.py + p1.y }, { x: a.px + p2.x, y: a.py + p2.y }, { x: b.px, y: b.py }];
}

function length(p: Pt[]) {
  let s = 0;
  for (let i = 1; i < p.length; i++) s += Math.hypot(p[i]!.x - p[i - 1]!.x, p[i]!.y - p[i - 1]!.y);
  return s;
}

/** `k` points evenly spaced along the path by length. */
function resample(p: Pt[], k: number): Pt[] {
  const total = length(p), out: Pt[] = [];
  let i = 1, walked = 0;
  for (let j = 0; j < k; j++) {
    const want = (total * j) / (k - 1);
    while (i < p.length - 1 && walked + Math.hypot(p[i]!.x - p[i - 1]!.x, p[i]!.y - p[i - 1]!.y) < want) {
      walked += Math.hypot(p[i]!.x - p[i - 1]!.x, p[i]!.y - p[i - 1]!.y); i++;
    }
    const a = p[i - 1]!, b = p[i]!, seg = Math.hypot(b.x - a.x, b.y - a.y) || 1, f = Math.min(1, Math.max(0, (want - walked) / seg));
    out.push({ x: a.x + (b.x - a.x) * f, y: a.y + (b.y - a.y) * f });
  }
  return out;
}

function mulberry32(seed: number) {
  let s = seed >>> 0;
  return () => {
    s = (s + 0x6d2b79f5) >>> 0;
    let r = Math.imul(s ^ (s >>> 15), 1 | s);
    r = (r + Math.imul(r ^ (r >>> 7), 61 | r)) ^ r;
    return ((r ^ (r >>> 14)) >>> 0) / 4294967296;
  };
}
