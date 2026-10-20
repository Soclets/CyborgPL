// Draws one frame of the model: the living network in white, the board it turns into in grey, and
// spikes that warm from white to the site's orange as their pathway hardens. When the board is
// formed, one orange pulse ripples out across it from the middle (owner, 2026-10-20: "Make it pulse
// orange when it becomes formed").

import { FADE, type Edge, type Pt, type Sim } from "./sim";

const PULSE = 110;                // ms one part glows for
const RIPPLE = 70;                // ms the pulse takes to go from the middle to the corners

const ORANGE = [255, 122, 61];

const ease = (m: number) => m * m * (3 - 2 * m);
const mix = (a: number, b: number, m: number) => a + (b - a) * m;
const glow = (m: number, alpha: number) => `rgba(${Math.round(mix(255, ORANGE[0]!, m))},${Math.round(mix(255, ORANGE[1]!, m))},${Math.round(mix(255, ORANGE[2]!, m))},${alpha})`;

function at(e: Edge, m: number, i: number): Pt {
  const a = e.organic[i]!, b = e.trace[i]!;
  return { x: mix(a.x, b.x, m), y: mix(a.y, b.y, m) };
}

/** The point a fraction `f` of the way along an edge, as it is drawn now. */
function along(e: Edge, m: number, f: number): Pt {
  const s = f * (e.organic.length - 1), i = Math.min(e.organic.length - 2, Math.floor(s));
  const a = at(e, m, i), b = at(e, m, i + 1), r = s - i;
  return { x: mix(a.x, b.x, r), y: mix(a.y, b.y, r) };
}

/** How bright the formed pulse is at (x, y) now, 0 to 1. */
function pulse(sim: Sim, x: number, y: number, w: number, h: number) {
  if (sim.formedAt === null) return 0;
  const far = Math.hypot(x - w / 2, y - h / 2) / Math.hypot(w / 2, h / 2);
  const f = (sim.t - sim.formedAt - far * RIPPLE) / PULSE;
  return f > 0 && f < 1 ? Math.sin(Math.PI * f) : 0;
}

export function draw(ctx: CanvasRenderingContext2D, sim: Sim, w: number, h: number) {
  ctx.clearRect(0, 0, w, h);
  ctx.globalAlpha = Math.min(1, Math.max(0, (sim.end - sim.t) / FADE));
  ctx.lineCap = "round";
  ctx.lineJoin = "round";

  // Axons that are still growing, and ones pulling back.
  ctx.lineWidth = 1;
  for (const c of sim.cones) {
    if (c.path.length < 2) continue;
    ctx.strokeStyle = c.dying ? "rgba(255,255,255,.07)" : "rgba(255,255,255,.16)";
    ctx.beginPath();
    ctx.moveTo(c.path[0]!.x, c.path[0]!.y);
    for (let i = 1; i < c.path.length; i++) ctx.lineTo(c.path[i]!.x, c.path[i]!.y);
    ctx.stroke();
    if (!c.dying) {
      const tip = c.path[c.path.length - 1]!;
      ctx.fillStyle = "rgba(255,255,255,.45)";
      ctx.fillRect(tip.x - 1, tip.y - 1, 2, 2);
    }
  }

  // Connections, from axon to trace.
  for (const e of sim.edges) {
    const m = ease(e.morph), mid = e.trace[e.trace.length >> 1]!, p = pulse(sim, mid.x, mid.y, w, h);
    ctx.lineWidth = mix(1, 1.6, m) + p;
    ctx.lineJoin = m > 0.5 ? "miter" : "round";
    ctx.strokeStyle = p ? glow(p, mix(0.2, 0.75, p)) : `rgba(255,255,255,${mix(0.16, 0.2, m)})`;
    ctx.beginPath();
    for (let i = 0; i < e.organic.length; i++) {
      const q = at(e, m, i);
      if (i) ctx.lineTo(q.x, q.y); else ctx.moveTo(q.x, q.y);
    }
    ctx.stroke();
  }

  // Spikes on their way.
  ctx.globalCompositeOperation = "lighter";
  for (const p of sim.pulses) {
    const m = ease(p.edge.morph), q = along(p.edge, m, (sim.t - p.at) / p.edge.delay);
    ctx.fillStyle = glow(m, 0.14);
    ctx.beginPath(); ctx.arc(q.x, q.y, 5, 0, Math.PI * 2); ctx.fill();
    ctx.fillStyle = glow(m, 0.9);
    ctx.beginPath(); ctx.arc(q.x, q.y, 1.5, 0, Math.PI * 2); ctx.fill();
  }

  // Cell bodies, from round to pads, lighting up when they fire.
  for (const n of sim.neurons) {
    if (sim.t < n.bornAt) continue;
    const born = Math.min(1, (sim.t - n.bornAt) / 80), m = ease(n.hard);
    const x = mix(n.x, n.px, m), y = mix(n.y, n.py, m);
    if (n.flash > 0.02) {
      ctx.fillStyle = glow(m, 0.3 * n.flash * born);
      ctx.beginPath(); ctx.arc(x, y, 4 + 9 * n.flash, 0, Math.PI * 2); ctx.fill();
    }
    const p = pulse(sim, x, y, w, h);
    if (p) {
      ctx.fillStyle = glow(1, 0.35 * p);
      ctx.beginPath(); ctx.arc(x, y, 6 + 8 * p, 0, Math.PI * 2); ctx.fill();
    }
  }
  ctx.globalCompositeOperation = "source-over";
  for (const n of sim.neurons) {
    if (sim.t < n.bornAt) continue;
    const born = Math.min(1, (sim.t - n.bornAt) / 80), m = ease(n.hard);
    const x = mix(n.x, n.px, m), y = mix(n.y, n.py, m), size = mix(6, 9, m), r = mix(3, 1.5, m);
    ctx.fillStyle = `rgba(${Math.round(mix(255, 11, m))},${Math.round(mix(255, 11, m))},${Math.round(mix(255, 11, m))},${mix(0.55, 1, m) * born})`;
    const p = pulse(sim, x, y, w, h);
    ctx.strokeStyle = p ? glow(p, mix(0.4, 0.95, p)) : `rgba(255,255,255,${0.4 * m * born})`;
    ctx.lineWidth = 1.2;
    ctx.beginPath(); ctx.roundRect(x - size / 2, y - size / 2, size, size, r); ctx.fill();
    if (m > 0.01) ctx.stroke();
    if (m > 0.5) {
      // The pad's drill hole.
      ctx.fillStyle = `rgba(255,255,255,${0.35 * (m - 0.5) * 2 * born})`;
      ctx.beginPath(); ctx.arc(x, y, 1.2, 0, Math.PI * 2); ctx.fill();
    }
  }
  ctx.globalAlpha = 1;
}
