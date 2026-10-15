import { useEffect, useRef } from "react";

export const meta = () => [
  { title: "Zag" },
  { name: "description", content: "Zag, a programming language: very slow at the development, fast & safe at the final product." },
];

// The sky (owner, 2026-10-15): "You land on the website, it has that blue sky with clouds
// background moving, and with a message 'Zag, a Zig ripoff? Hm, maybe not...'"
// Three layers of clouds drift across at different speeds, the far ones small, faint and slow.
// [layer, top %, width px, seconds to cross, how far along it starts (0–1)]
const CLOUDS = [
  ["far", 8, 150, 170, 0.05], ["far", 18, 120, 190, 0.42], ["far", 12, 170, 160, 0.71], ["far", 26, 130, 200, 0.9],
  ["mid", 34, 230, 110, 0.18], ["mid", 46, 270, 125, 0.55], ["mid", 40, 210, 100, 0.83],
  ["near", 62, 380, 70, 0.3], ["near", 74, 440, 80, 0.68], ["near", 68, 340, 65, 0.97],
];

function Cloud({ id }) {
  // Puffs on a soft base, with a pale underside, as the logo's cloud is built. The colours are
  // CSS variables, so the clouds darken as the storm comes in.
  const puffs = (
    <>
      <circle cx="70" cy="64" r="34" /><circle cx="122" cy="44" r="44" /><circle cx="176" cy="62" r="32" />
      <circle cx="48" cy="92" r="28" /><circle cx="100" cy="104" r="28" /><circle cx="152" cy="104" r="30" /><circle cx="200" cy="92" r="26" />
      <ellipse cx="124" cy="84" rx="96" ry="34" />
    </>
  );
  return (
    <svg viewBox="0 0 248 140" aria-hidden="true">
      <defs><clipPath id={`c${id}`}>{puffs}</clipPath></defs>
      <g className="shade">{puffs}</g>
      <g clipPath={`url(#c${id})`}><g className="lit" transform="translate(0,-9)">{puffs}</g></g>
    </svg>
  );
}

// The screens move one at a time inside a fixed frame (owner, 2026-10-15: "Each scroll should be
// auto perfect in this area", then "There's still rubber-banding, and some scrolls don't
// respond"). The document itself never scrolls, so Safari has nothing to bounce, and every
// gesture moves exactly one screen: a wheel or trackpad swipe, a touch swipe, or the keys.
// A screen taller than the window scrolls inside first.
const NEXT = ["ArrowDown", "PageDown", " "], PREV = ["ArrowUp", "PageUp"];
function usePager(frame) {
  useEffect(() => {
    const box = frame.current;
    const screens = () => [...box.children];
    const still = matchMedia("(prefers-reduced-motion: reduce)");
    let at = 0, lockedUntil = 0, lastDir = 0, lastWheel = 0, touchY = null, recent = [], raf = 0;
    // The move itself: starts at full speed and eases out, so the page answers at once (owner:
    // "it feels like there's a delay before a scroll registers"; the browser's smooth scroll
    // started slowly). A new move takes over from wherever the last one is.
    const glide = (to) => {
      cancelAnimationFrame(raf);
      if (still.matches) { box.scrollTop = to; return; }
      const from = box.scrollTop, t0 = performance.now(), length = 460;
      const step = (t) => {
        const p = Math.min(1, (t - t0) / length);
        box.scrollTop = from + (to - from) * (1 - Math.pow(1 - p, 3));
        if (p < 1) raf = requestAnimationFrame(step);
      };
      raf = requestAnimationFrame(step);
    };
    const go = (i) => {
      const list = screens();
      i = Math.max(0, Math.min(list.length - 1, i));
      if (i === at) return;
      lastDir = Math.sign(i - at);
      at = i;
      // Only a little while, and only for the same way: turning back works at any moment.
      lockedUntil = performance.now() + 240;
      glide(list[i].offsetTop);
    };
    const ready = (dir) => performance.now() >= lockedUntil || dir !== lastDir;
    // Can the current screen still scroll this way by itself?
    const inner = (dir) => {
      const el = screens()[at];
      return dir > 0 ? el.scrollTop + el.clientHeight < el.scrollHeight - 1 : el.scrollTop > 0;
    };
    const onWheel = (e) => {
      const dir = Math.sign(e.deltaY);
      if (!dir || inner(dir)) return;
      e.preventDefault();
      // A trackpad keeps sending wheel events for a second or two as it coasts, each smaller
      // than the last. A new swipe is told apart by coming after a pause, or by speeding up
      // again; a swipe made while the last one still coasts used to be ignored until a click
      // stopped the coasting (owner: "only the first scroll works, unless you click").
      const now = performance.now(), size = Math.abs(e.deltaY);
      const paused = now - lastWheel > 180;
      const before = recent.length ? recent.reduce((a, b) => a + b, 0) / recent.length : 0;
      const faster = recent.length >= 2 && size > 6 && size > before * 1.5;
      lastWheel = now;
      recent = paused ? [size] : [...recent.slice(-4), size];
      if (!ready(dir) || !(paused || faster)) return;
      recent = [size];
      go(at + dir);
    };
    const onTouchStart = (e) => { touchY = e.touches[0].clientY; };
    const onTouchMove = (e) => {
      if (touchY === null) return;
      const dir = Math.sign(touchY - e.touches[0].clientY);
      if (dir && !inner(dir)) e.preventDefault();
    };
    const onTouchEnd = (e) => {
      if (touchY === null) return;
      const dy = touchY - e.changedTouches[0].clientY;
      touchY = null;
      if (Math.abs(dy) > 30 && !inner(Math.sign(dy)) && ready(Math.sign(dy))) go(at + Math.sign(dy));
    };
    const onKey = (e) => {
      if (e.target.closest?.("input, textarea, select, [contenteditable]")) return;
      const dir = NEXT.includes(e.key) && !e.shiftKey ? 1 : PREV.includes(e.key) || (e.key === " " && e.shiftKey) ? -1 : 0;
      if (dir) { if (inner(dir)) return; e.preventDefault(); go(at + dir); }
      else if (e.key === "Home") { e.preventDefault(); go(0); }
      else if (e.key === "End") { e.preventDefault(); go(screens().length - 1); }
    };
    const onResize = () => { cancelAnimationFrame(raf); box.scrollTop = screens()[at].offsetTop; };
    // On the window, not the frame: Safari only sent the frame its wheel events after a click
    // on the page (owner: "you have to click the screen before scrolling"). The frame also takes
    // focus, so the keys work straight away too.
    addEventListener("wheel", onWheel, { passive: false });
    addEventListener("touchstart", onTouchStart, { passive: true });
    addEventListener("touchmove", onTouchMove, { passive: false });
    addEventListener("touchend", onTouchEnd);
    addEventListener("keydown", onKey);
    addEventListener("resize", onResize);
    box.focus({ preventScroll: true });
    return () => {
      removeEventListener("wheel", onWheel); removeEventListener("touchstart", onTouchStart);
      removeEventListener("touchmove", onTouchMove); removeEventListener("touchend", onTouchEnd);
      removeEventListener("keydown", onKey); removeEventListener("resize", onResize); cancelAnimationFrame(raf);
    };
  }, [frame]);
}

// Scrolling down turns the sky to storm (owner: "Zomp comes in with a lightning strike, and the
// background turns storm"), and on to the site the storm clears again. --storm runs from 0 (sky)
// to 1 (storm) as the Zomp screen comes up, and back to 0 as the site's screen does; the strike
// plays each time Zomp arrives.
function useStorm(frame, zomp, site) {
  useEffect(() => {
    const root = document.documentElement, box = frame.current;
    let raf = 0;
    const rise = (el) => {
      const r = el.getBoundingClientRect(), h = innerHeight;
      return Math.min(1, Math.max(0, (h - r.top) / (h * 0.75)));
    };
    const update = () => {
      raf = 0;
      root.style.setProperty("--storm", Math.max(0, rise(zomp.current) - rise(site.current)).toFixed(3));
    };
    const onScroll = () => { if (!raf) raf = requestAnimationFrame(update); };
    update();
    box.addEventListener("scroll", onScroll, { passive: true });
    addEventListener("resize", onScroll);
    const seen = new IntersectionObserver(([e]) => {
      if (e.isIntersecting) root.classList.add("struck");
      else if (e.boundingClientRect.top > 0) root.classList.remove("struck");
    }, { threshold: 0.45 });
    seen.observe(zomp.current);
    return () => {
      box.removeEventListener("scroll", onScroll); removeEventListener("resize", onScroll);
      seen.disconnect(); cancelAnimationFrame(raf); root.classList.remove("struck");
    };
  }, [frame, zomp, site]);
}

// The site's top bar: a floating pill (owner: "I think it should be an in-theme pill").
const LINKS = [["Learn", "/learn"], ["Examples", "/examples"], ["Download", "/download"], ["Licence", "/licence"], ["Commits", "/commits"]];
function Pill() {
  return (
    <nav className="pill cloud" aria-label="Site">
      <a className="brand" href="/"><img src="/logo.svg" alt="Zag" width="45" height="34" /></a>
      <div className="links">
        {LINKS.map(([name, to]) => <a key={to} className="link" href={to}>{name}</a>)}
      </div>
    </nav>
  );
}

export default function Home() {
  const frame = useRef(null), zomp = useRef(null), site = useRef(null);
  usePager(frame);
  useStorm(frame, zomp, site);
  return (
    <>
      <div className="backdrop" aria-hidden="true">
        <div className="storm" />
        <div className="clouds">
          {CLOUDS.map(([layer, top, w, s, at], i) => (
            <div key={i} className={`cloud ${layer}`}
              style={{ top: `${top}%`, "--w": `${w}px`, animationDuration: `${s}s`, animationDelay: `${-at * s}s` }}>
              <Cloud id={i} />
            </div>
          ))}
        </div>
        <div className="flash" />
      </div>
      <main className="frame" ref={frame} tabIndex={-1}>
        <section className="hero">
          <img className="logo" src="/logo.svg" alt="Zag" width="392" height="294" />
          <h1>Zag, a Zig ripoff?</h1>
          <p className="aside">Hm, maybe not...</p>
        </section>
        <section className="zomp" ref={zomp}>
          <div className="title">
            <svg className="bolt" viewBox="0 0 120 600" preserveAspectRatio="none" aria-hidden="true">
              <polyline pathLength="1" points="64,0 46,120 78,150 40,290 82,322 50,460 70,600" />
            </svg>
            <h2>Zomp</h2>
          </div>
          <p className="what">Zag’s compiler.</p>
          <p>Zomp reads your whole program and proves, before it ever runs, that no number overflows and no index runs past the end. The final build carries none of those checks, or it doesn’t build.</p>
          <p>It takes its time: every build is a full one, with no fast mode. What comes out is native code, through LLVM.</p>
          <p className="cmd"><code>zomp</code></p>
        </section>
        <section className="site" ref={site}>
          <Pill />
          <div className="site-top">
            <h2>Very slow at the development, fast &amp; safe at the final product.</h2>
          </div>
        </section>
      </main>
    </>
  );
}
