import { useEffect, useRef, useState } from "react";
import { Link, useLoaderData } from "react-router";
import { SITE_SCREEN } from "../sky.jsx";
import { BUILT_WITH, COMPARED, COMPARISON, CONS, CORE, HOME_EXAMPLES, PROS } from "../content.js";
import Icon from "../icons.jsx";
import Md from "../md.jsx";

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
    let at = 0, lockedUntil = 0, lastDir = 0, lastWheel = 0, touchY = null, raf = 0;
    // The swipe's smoothed speed, and since the last move: its fastest, whether it has started
    // to slow, and its slowest since then.
    let speed = 0, peak = 0, slowed = true, trough = 0;
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
      // A while, and only for the same way: turning back works at any moment.
      lockedUntil = performance.now() + 450;
      const length = glide(list[i].offsetTop, away);
      // Arrive a little before the move ends, so the strike lands as Zomp settles in.
      clearTimeout(arrival);
      arrival = setTimeout(() => arrive(i, was), length * 0.45);
    };
    const ready = (dir) => performance.now() >= lockedUntil || dir !== lastDir;
    // Can the current screen still scroll this way by itself?
    // Not while its content is still drifting into place: the drift pokes past the screen's
    // edge and would look like more to scroll through.
    // So can a panel inside it under the pointer, the examples' code for one: it scrolls first.
    const inner = (dir, target) => {
      const el = screens()[at];
      if (Math.abs(parseFloat(el.style.getPropertyValue("--d")) || 0) > 0.01) return false;
      const can = (x) => dir > 0 ? x.scrollTop + x.clientHeight < x.scrollHeight - 1 : x.scrollTop > 0;
      for (let x = target instanceof Element ? target : null; x && x !== el && el.contains(x); x = x.parentElement) {
        const y = getComputedStyle(x).overflowY;
        if ((y === "auto" || y === "scroll") && x.scrollHeight > x.clientHeight && can(x)) return true;
      }
      return can(el);
    };
    const onWheel = (e) => {
      const dir = Math.sign(e.deltaY);
      if (!dir || inner(dir, e.target)) return;
      e.preventDefault();
      // A mouse wheel and a trackpad are told apart (owner: "you could sniff if mouse or
      // trackpad"), and a wheel click, having no coasting to see through, is simply a move once
      // the last move the same way has had its moment. Only clear signs count as a wheel: Firefox
      // counting in lines, or Safari's and Chrome's old wheelDeltaY in whole notches of 120 that
      // aren't a trackpad's -3 times deltaY. Anything less clear is read as a trackpad, which
      // works for a wheel too, a little more cautiously.
      const w = e.wheelDeltaY || 0;
      const wheel = e.deltaMode !== 0 || (w !== 0 && w % 120 === 0 && Math.abs(w + 3 * e.deltaY) >= 3);
      if (wheel) {
        lastWheel = performance.now();
        if (ready(dir)) go(at + dir);
        return;
      }
      // A trackpad keeps sending wheel events for a second or two as it coasts, each smaller
      // than the last. A new swipe is told apart by coming after a pause, or by speeding up
      // again; a swipe made while the last one still coasts used to be ignored until a click
      // stopped the coasting (owner: "only the first scroll works, unless you click").
      // One swipe is one move (owner: "scroll could be somehow two", then "still here. But not
      // so reproducible"). A trackpad's events are judged by their smoothed speed, not one by
      // one, so a single uneven event can't pass for a new swipe; and a gap in them counts as the
      // pause before a new swipe only when it is long, because Safari can hold events back for a
      // moment while it draws a move.
      const now = performance.now(), size = Math.abs(e.deltaY), gap = now - lastWheel;
      lastWheel = now;
      if (gap > 350) { speed = size; peak = size; trough = size; slowed = true; }
      else speed = speed * 0.6 + size * 0.4;
      peak = Math.max(peak, speed);
      if (speed < peak * 0.6) slowed = true;
      trough = slowed ? Math.min(trough, speed) : speed;
      // A long gap is a new swipe; a short one, even at the coast's tail, is only Safari busy
      // drawing (a new swipe then shows by speeding up again, below).
      const fresh = gap > 350;
      // A swipe made while the last one still coasts: it speeds up again after slowing down.
      const again = slowed && speed > 10 && speed > trough * 1.8 && size > trough * 2.2;
      if (!(fresh || again) || !ready(dir)) return;
      peak = speed; trough = speed; slowed = false;
      go(at + dir);
    };
    let touchAt = null;
    const onTouchStart = (e) => { touchY = e.touches[0].clientY; touchAt = e.target; };
    const onTouchMove = (e) => {
      if (touchY === null) return;
      const dir = Math.sign(touchY - e.touches[0].clientY);
      if (dir && !inner(dir, touchAt)) e.preventDefault();
    };
    const onTouchEnd = (e) => {
      if (touchY === null) return;
      const dy = touchY - e.changedTouches[0].clientY;
      touchY = null;
      if (Math.abs(dy) > 30 && !inner(Math.sign(dy), touchAt) && ready(Math.sign(dy))) go(at + Math.sign(dy));
    };
    const onKey = (e) => {
      if (e.target.closest?.("input, textarea, select, [contenteditable]")) return;
      const dir = NEXT.includes(e.key) && !e.shiftKey ? 1 : PREV.includes(e.key) || (e.key === " " && e.shiftKey) ? -1 : 0;
      // A screen with more than fits pages through it first, by most of a window at a time.
      if (dir) {
        e.preventDefault();
        if (inner(dir)) screens()[at].scrollBy({ top: dir * box.clientHeight * 0.8, behavior: still.matches ? "auto" : "smooth" });
        else go(at + dir);
      }
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

// At build time: each example's source, highlighted with the site's grammar, and its output.
export async function loader() {
  const { readFileSync } = await import("node:fs");
  const path = await import("node:path");
  const { createHighlighter } = await import("shiki");
  const { ZAG_THEME } = await import("../zag-theme.js");
  const grammar = JSON.parse(readFileSync(path.resolve("../grammar/nerve.tmLanguage.json"), "utf8"));
  const hl = await createHighlighter({ themes: [ZAG_THEME], langs: [grammar] });
  const dir = path.resolve("../../examples");
  return {
    examples: HOME_EXAMPLES.map((e) => {
      const source = readFileSync(path.join(dir, `${e.id}.zg`), "utf8").replace(/\n$/, "");
      const lines = source.split("\n").length;
      return { ...e, note: e.note.replace("{lines}", String(lines)), file: `${e.id}.zg`, html: hl.codeToHtml(source, { lang: "nerve", theme: "zag-night" }), out: readFileSync(path.join(dir, `${e.id}.out`), "utf8").replace(/\n$/, "") };
    }),
  };
}

// The examples, one at a time behind tabs whose highlight slides as the pill's does: the code
// full width, Zag's lines being long, and what it prints under it.
function Examples({ examples }) {
  const [on, setOn] = useState(0);
  const tabs = useRef(null), thumb = useRef(null);
  useEffect(() => {
    const place = () => {
      const b = tabs.current?.children[on + 1];
      if (!b) return;
      thumb.current.style.setProperty("--x", `${b.offsetLeft}px`);
      thumb.current.style.setProperty("--w", `${b.offsetWidth}px`);
    };
    place();
    document.fonts?.ready.then(place);
    addEventListener("resize", place);
    return () => removeEventListener("resize", place);
  }, [on]);
  const ex = examples[on];
  return (
    <section className="ex" aria-labelledby="ex-title">
      <h3 id="ex-title">Examples</h3>
      <div className="ex-tabs" ref={tabs} role="tablist">
        <span className="ex-thumb" ref={thumb} aria-hidden="true" />
        {examples.map((e, i) => (
          <button key={e.id} role="tab" aria-selected={i === on} className="ex-tab" onClick={() => setOn(i)}>{e.title}</button>
        ))}
      </div>
      <div className="ex-card" role="tabpanel">
        <p className="ex-note"><Md text={ex.note} /></p>
        <div className="ex-panes">
          <figure className="ex-code">
            <figcaption>{ex.file}</figcaption>
            <div dangerouslySetInnerHTML={{ __html: ex.html }} />
          </figure>
          <figure className="ex-out">
            <figcaption>Output</figcaption>
            <pre>{ex.out}</pre>
          </figure>
        </div>
      </div>
      <Link className="ex-all" to="/examples">All examples</Link>
    </section>
  );
}

export default function Home() {
  const { examples } = useLoaderData();
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
          <div className="moves site-body">
            <div className="site-top">
              <h2>Very slow at the development, fast &amp; safe at the final product.</h2>
            </div>
            {/* The core features (owner, 2026-10-15: "let's expand the Home. The Core Features"). */}
            <section className="core" aria-labelledby="core-title">
              <h3 id="core-title">Core features</h3>
              <ul className="cards">
                {CORE.map((f) => (
                  <li key={f.title} className="card">
                    <span className="card-icon"><Icon name={f.icon} /></span>
                    <h4><Md text={f.title} /></h4>
                    <p><Md text={f.text} /></p>
                  </li>
                ))}
              </ul>
            </section>
            <Examples examples={examples} />
            {/* Pros and cons: two lists, not pairs (owner). */}
            <section className="lists" aria-label="Pros and cons">
              {[["Pros", PROS, "pros"], ["Cons", CONS, "cons"]].map(([title, items, kind]) => (
                <div key={kind} className={`list-card ${kind}`}>
                  <h3>{title}</h3>
                  <ul>{items.map((t) => <li key={t}><Md text={t} /></li>)}</ul>
                </div>
              ))}
            </section>
            {/* The comparison (owner, 2026-10-15): Zag beside other languages' latest stable versions. */}
            <section className="compare" aria-labelledby="compare-title">
              <h3 id="compare-title">Zag beside other languages</h3>
              <div className="compare-card">
                <div className="compare-scroll">
                  <table className="compare-table">
                    <thead>
                      <tr>
                        <th scope="col"><span className="sr-only">Topic</span></th>
                        {COMPARED.map((l) => (
                          <th key={l.id} scope="col" className={l.id === "zag" ? "ours" : undefined}>
                            <b>{l.name}</b>
                            <small>{l.url ? <a href={l.url} target="_blank" rel="noopener noreferrer">{l.version}</a> : l.version}</small>
                          </th>
                        ))}
                      </tr>
                    </thead>
                    <tbody>
                      {COMPARISON.map((r) => (
                        <tr key={r.topic}>
                          <th scope="row">{r.topic}</th>
                          {COMPARED.map((l) => <td key={l.id} className={l.id === "zag" ? "ours" : undefined}><Md text={r[l.id]} /></td>)}
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
                <p className="compare-note">Checked 15 Oct 2026: versions from each language’s official release index; Rust from the <a href="https://doc.rust-lang.org/book/" target="_blank" rel="noopener noreferrer">Rust Book</a> and <a href="https://doc.rust-lang.org/reference/" target="_blank" rel="noopener noreferrer">Reference</a>, Zig from its <a href="https://ziglang.org/documentation/0.17.0/" target="_blank" rel="noopener noreferrer">0.17.0 language reference</a>, Zag from its design notes. Stated behaviour, not benchmarks.</p>
              </div>
            </section>
            {/* What Zag's own tools are made with (owner, 2026-10-15: "Yeah"). */}
            <section className="built" aria-labelledby="built-title">
              <h3 id="built-title">Built with</h3>
              <table className="built-table">
                <thead><tr><th scope="col">Part</th><th scope="col">Made with</th></tr></thead>
                <tbody>
                  {BUILT_WITH.map((b) => (
                    <tr key={b.part}><td><Md text={b.part} /></td><td><Md text={b.made} /></td></tr>
                  ))}
                </tbody>
              </table>
            </section>
            {/* The licence, credits and trademarks live on More (owner: "Shouldn't that be for More?"). */}
            <p className="home-end">The licence, credits and trademarks are on <Link to="/more">More</Link>.</p>
          </div>
        </section>
      </main>
    </>
  );
}
