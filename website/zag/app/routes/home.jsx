import { useEffect, useRef } from "react";
import { SITE_SCREEN } from "../sky.jsx";

export const meta = () => [
  { title: "Zag" },
  { name: "description", content: "Zag, a programming language: very slow at the development, fast & safe at the final product." },
];

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
    const index = (id) => screens().findIndex((el) => el.id === id);
    const still = matchMedia("(prefers-reduced-motion: reduce)");
    let at = 0, lockedUntil = 0, lastDir = 0, lastWheel = 0, touchY = null, recent = [], raf = 0;
    // Since the last move: the swipe's biggest step so far, and whether it has started slowing.
    let peak = 0, slowed = true;
    // The move itself: starts at full speed and eases out, so the page answers at once (owner:
    // "it feels like there's a delay before a scroll registers"; the browser's smooth scroll
    // started slowly). A new move takes over from wherever the last one is.
    // It settles with a long, soft tail (owner: "Make the transitions good"), and a jump over
    // more than one screen takes a little longer instead of whipping past.
    let finish = 0, arrival = 0;
    const glide = (to, screensAway) => {
      cancelAnimationFrame(raf); clearTimeout(finish);
      if (still.matches) { box.scrollTop = to; return 0; }
      const from = box.scrollTop, t0 = performance.now(), length = 620 + 200 * (screensAway - 1);
      const settle = () => { box.scrollTop = to; box.dispatchEvent(new Event("settled")); };
      const step = (t) => {
        const p = Math.min(1, (t - t0) / length);
        box.scrollTop = from + (to - from) * (1 - Math.pow(1 - p, 4));
        if (p < 1) raf = requestAnimationFrame(step); else settle();
      };
      raf = requestAnimationFrame(step);
      // If the browser stops drawing frames part-way (a hidden tab), still end on the screen.
      finish = setTimeout(() => { cancelAnimationFrame(raf); settle(); }, length + 200);
      return length;
    };
    // The page knows which screen it is on: the lightning strikes only when Zomp is where you
    // land, not when a jump passes through it, and plays again after you go back up to the sky.
    const root = document.documentElement;
    // It strikes only when you come down to Zomp; coming up to it from below, Zomp is simply
    // there (owner: "refreshing could trick the lightning to strike on Zomp, even though it's a
    // scroll-up").
    const arrive = (i, from) => {
      const id = screens()[i].id, z = index("zomp");
      root.dataset.screen = id;
      if (id === "zomp") root.classList.add(from < z ? "struck" : "calm");
      else if (i < z) root.classList.remove("struck", "calm");
    };
    const go = (i) => {
      const list = screens();
      i = Math.max(0, Math.min(list.length - 1, i));
      if (i === at) return;
      const away = Math.abs(i - at), was = at;
      lastDir = Math.sign(i - at);
      at = i;
      // Only a little while, and only for the same way: turning back works at any moment.
      lockedUntil = performance.now() + 240;
      const length = glide(list[i].offsetTop, away);
      // Arrive a little before the move ends, so the strike lands as Zomp settles in.
      clearTimeout(arrival);
      arrival = setTimeout(() => arrive(i, was), length * 0.45);
    };
    const ready = (dir) => performance.now() >= lockedUntil || dir !== lastDir;
    // Can the current screen still scroll this way by itself?
    // Not while its content is still drifting into place: the drift pokes past the screen's
    // edge and would look like more to scroll through.
    const inner = (dir) => {
      const el = screens()[at];
      if (Math.abs(parseFloat(el.style.getPropertyValue("--d")) || 0) > 0.01) return false;
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
      // A swipe moves the page on its first event but keeps speeding up for a moment after; that
      // isn't a second swipe (owner: "scroll could be somehow two"). Speeding up only counts once
      // this swipe has started to slow down.
      peak = Math.max(peak, size);
      if (size < peak * 0.7) slowed = true;
      const faster = slowed && recent.length >= 2 && size > 6 && size > before * 1.5;
      lastWheel = now;
      recent = paused ? [size] : [...recent.slice(-4), size];
      if (!ready(dir) || !(paused || faster)) return;
      recent = [size];
      peak = size; slowed = false;
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
    const onResize = () => { cancelAnimationFrame(raf); clearTimeout(finish); box.scrollTop = screens()[at].offsetTop; };
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
    // A screen by its id: /#home opens on the site's own screen, and a link to it on this page
    // glides there in place of reloading.
    const start = index(location.hash.slice(1));
    // Coming from another page of the site (which sits at the site screen's height): with no
    // screen named, glide up from there to the start, so the sky moves rather than jumps.
    const from = parseFloat(root.style.getPropertyValue("--y")) || 0;
    if (start > 0) at = start;
    else if (start < 0 && Math.round(from) === SITE_SCREEN) {
      at = SITE_SCREEN;
      requestAnimationFrame(() => go(0));
    }
    // Always open exactly on that screen: Safari may restore where the frame was scrolled to
    // before a reload, which left the page between screens and unsure of where it was.
    box.scrollTop = screens()[at].offsetTop;
    box.dispatchEvent(new Event("settled"));
    arrive(at, at);
    const onClick = (e) => {
      const a = e.target.closest?.("a[href]");
      if (!a || e.metaKey || e.ctrlKey || e.shiftKey || e.button) return;
      const url = new URL(a.href, location.href);
      if (url.origin !== location.origin || url.pathname !== location.pathname) return;
      const i = url.hash ? index(url.hash.slice(1)) : 0;
      if (i < 0) return;
      e.preventDefault();
      history.replaceState(null, "", url.hash || location.pathname);
      go(i);
    };
    addEventListener("click", onClick, true);
    return () => {
      removeEventListener("wheel", onWheel); removeEventListener("touchstart", onTouchStart);
      removeEventListener("touchmove", onTouchMove); removeEventListener("touchend", onTouchEnd);
      removeEventListener("keydown", onKey); removeEventListener("resize", onResize); removeEventListener("click", onClick, true); cancelAnimationFrame(raf); clearTimeout(finish); clearTimeout(arrival);
      root.classList.remove("struck", "calm"); delete root.dataset.screen;
    };
  }, [frame]);
}

// As the page moves (owner: "Make the transitions good"), every frame sets:
// - --storm: 0 (sky) to 1 (storm) as the Zomp screen comes up, back to 0 as the site's screen
//   does (owner: "Zomp comes in with a lightning strike, and the background turns storm");
// - --y: how many screens down the page is, which moves the cloud layers at their own speeds;
// - on each screen, --d (where it is: 0 in place, 1 a screen below, -1 a screen above) and
//   --fade, so its content drifts and fades as it leaves and rises into place as it comes.
function useMotion(frame, zomp, site) {
  useEffect(() => {
    const root = document.documentElement, box = frame.current;
    let raf = 0;
    const update = () => {
      raf = 0;
      const h = box.clientHeight, y = box.scrollTop / h;
      const rise = (el) => Math.min(1, Math.max(0, (h - (el.offsetTop - box.scrollTop)) / (h * 0.75)));
      root.style.setProperty("--storm", Math.max(0, rise(zomp.current) - rise(site.current)).toFixed(3));
      root.style.setProperty("--y", y.toFixed(4));
      for (const el of box.children) {
        const d = (el.offsetTop - box.scrollTop) / h;
        el.style.setProperty("--d", d.toFixed(4));
        el.style.setProperty("--fade", Math.max(0, 1 - Math.abs(d) * 1.35).toFixed(4));
      }
      // The pill belongs to the site screen: it fades in with it, and can't be clicked away from it.
      const pill = Math.max(0, 1 - Math.abs((site.current.offsetTop - box.scrollTop) / h) * 1.35);
      root.style.setProperty("--site-fade", pill.toFixed(4));
      root.classList.toggle("pill-off", pill < 0.05);
    };
    const onScroll = () => { if (!raf) raf = requestAnimationFrame(update); };
    update();
    root.classList.add("moving");
    // When a move ends, place everything at once, without waiting for the next frame.
    const onSettled = () => { cancelAnimationFrame(raf); update(); };
    box.addEventListener("settled", onSettled);
    box.addEventListener("scroll", onScroll, { passive: true });
    addEventListener("resize", onScroll);
    return () => {
      box.removeEventListener("scroll", onScroll); box.removeEventListener("settled", onSettled); removeEventListener("resize", onScroll); cancelAnimationFrame(raf);
      root.classList.remove("moving", "pill-off");
    };
  }, [frame, zomp, site]);
}

export default function Home() {
  const frame = useRef(null), zomp = useRef(null), site = useRef(null);
  usePager(frame);
  useMotion(frame, zomp, site);
  return (
    <>
      <main className="frame" ref={frame} tabIndex={-1}>
        <section className="hero" id="top">
          <div className="moves">
            <img className="logo" src="/logo.svg" alt="Zag" width="392" height="294" />
            <h1>Zag, a Zig ripoff?</h1>
            <p className="aside">Hm, maybe not...</p>
          </div>
        </section>
        <section className="zomp" id="zomp" ref={zomp}>
          <div className="moves">
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
          </div>
        </section>
        <section className="site" id="home" ref={site}>
          <div className="site-top moves">
            <h2>Very slow at the development, fast &amp; safe at the final product.</h2>
          </div>
        </section>
      </main>
    </>
  );
}
