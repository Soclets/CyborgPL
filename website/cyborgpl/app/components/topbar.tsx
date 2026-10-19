import { AnimatePresence, motion } from "motion/react";
import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { Link, useLocation } from "react-router";
import mark from "../assets/mark-white.svg?url";
import { NAV, PRIMARY, type NavLink } from "./nav";

// The topbar (owner, 2026-10-19, after x.ai's: "there's also a smooth animation when you hover
// slide over the buttons"). One highlight glides from item to item under the pointer (Motion's
// shared layout, layoutId); a menu opens under an item on hover, click or keyboard. On a phone the
// items fold into a menu button that opens a full-screen sheet.
const glide = { type: "spring", stiffness: 520, damping: 40, mass: 0.8 } as const;
const pop = { initial: { opacity: 0, y: -6, scale: 0.98 }, animate: { opacity: 1, y: 0, scale: 1 }, exit: { opacity: 0, y: -4, scale: 0.98 }, transition: { duration: 0.16, ease: "easeOut" } } as const;

function Chevron({ open }: { open?: boolean }) {
  return (
    <motion.svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true" animate={{ rotate: open ? 180 : 0 }} transition={glide}>
      <path d="m2 3.75 3 3 3-3" />
    </motion.svg>
  );
}

function Menu({ links, onPick, align = "left" }: { links: NavLink[]; onPick: () => void; align?: "left" | "right" }) {
  return (
    <motion.div className={`menu menu-${align}`} role="menu" {...pop}>
      {links.map((l) => (
        <Link key={l.to + l.label} className="menu-link" to={l.to} role="menuitem" onClick={onPick}>
          <span className="menu-label">{l.label}</span>
          {l.note && <span className="menu-note">{l.note}</span>}
        </Link>
      ))}
    </motion.div>
  );
}

// The menus share one panel (owner, 2026-10-19: "The dropdown things could be warped into the next
// thing in topbar"). Moving from one item to the next, the panel glides under the new item and
// takes its size, while the old links slide out and the new ones slide in, the way the pointer went.
function SharedMenu({ index, from, x, onPick }: { index: number; from: number; x: number; onPick: () => void }) {
  const item = NAV[index]!;
  const content = useRef<HTMLDivElement>(null);
  const [size, setSize] = useState<{ w: number; h: number } | null>(null);
  useLayoutEffect(() => {
    const el = content.current;
    if (!el) return;
    const measure = () => setSize({ w: el.offsetWidth, h: el.offsetHeight });
    measure();
    const ro = new ResizeObserver(measure);
    ro.observe(el);
    return () => ro.disconnect();
  }, [index]);
  const dir = Math.sign(index - from) || 0;
  // The outer layer carries the glide and bridges the gap under the bar with clear padding, so the
  // pointer never leaves the bar on its way down; the inner panel sizes itself and clips the slides.
  return (
    <motion.div
      className="menu-bridge"
      initial={{ opacity: 0, y: -6, scale: 0.98, x }}
      animate={{ opacity: 1, y: 0, scale: 1, x }}
      exit={{ opacity: 0, y: -4, scale: 0.98, transition: { duration: 0.14 } }}
      transition={{ ...glide, opacity: { duration: 0.16 } }}
    >
    <motion.div
      className="menu menu-shared"
      role="menu"
      animate={size ? { width: size.w, height: size.h } : undefined}
      transition={glide}
    >
      <AnimatePresence initial={false} mode="popLayout" custom={dir}>
        <motion.div
          key={item.label}
          ref={content}
          className="menu-content"
          custom={dir}
          variants={{
            enter: (d: number) => ({ opacity: 0, x: d * 48 }),
            center: { opacity: 1, x: 0 },
            leave: (d: number) => ({ opacity: 0, x: d * -48 }),
          }}
          initial="enter"
          animate="center"
          exit="leave"
          transition={{ ...glide, opacity: { duration: 0.18 } }}
        >
          {item.menu!.map((l) => {
            const inner = <><span className="menu-label">{l.label}{l.to.startsWith("https://") && <span className="menu-out" aria-hidden="true"> ↗</span>}</span>{l.note && <span className="menu-note">{l.note}</span>}</>;
            return l.to.startsWith("https://")
              ? <a key={l.to + l.label} className="menu-link" href={l.to} target="_blank" rel="noopener noreferrer" role="menuitem" onClick={onPick}>{inner}</a>
              : <Link key={l.to + l.label} className="menu-link" to={l.to} role="menuitem" onClick={onPick}>{inner}</Link>;
          })}
        </motion.div>
      </AnimatePresence>
    </motion.div>
    </motion.div>
  );
}

export function Topbar() {
  // One highlight: it sits under the item last pointed at, slides to the next (layoutId), and
  // fades out, staying put, when the pointer leaves the bar.
  const [hover, setHover] = useState<number | null>(null);
  const [last, setLast] = useState<number | null>(null);
  const [open, setOpen] = useState<number | "primary" | null>(null);
  const [sheet, setSheet] = useState(false);
  const bar = useRef<HTMLElement>(null);
  const items = useRef<(HTMLDivElement | null)[]>([]);
  const [from, setFrom] = useState(0);
  // Which menu the panel is coming from, so its contents slide the way the pointer moved.
  const openMenu = (i: number) => { setFrom(typeof open === "number" ? open : i); setOpen(i); };
  const closeTimer = useRef<ReturnType<typeof setTimeout>>(undefined);
  const { pathname } = useLocation();

  // A menu closes on a click elsewhere, on Escape, and when the page changes.
  useEffect(() => {
    const away = (e: PointerEvent) => { if (!bar.current?.contains(e.target as Node)) setOpen(null); };
    const esc = (e: KeyboardEvent) => { if (e.key === "Escape") { setOpen(null); setSheet(false); } };
    addEventListener("pointerdown", away);
    addEventListener("keydown", esc);
    return () => { removeEventListener("pointerdown", away); removeEventListener("keydown", esc); };
  }, []);
  useEffect(() => { setOpen(null); setSheet(false); }, [pathname]);
  useEffect(() => { document.documentElement.classList.toggle("sheet-open", sheet); }, [sheet]);

  // Hovering opens a menu at once and leaving closes it after a beat, so the pointer can travel
  // from the item down into its menu.
  const enter = (i: number) => { clearTimeout(closeTimer.current); setHover(i); setLast(i); if (NAV[i]!.menu) openMenu(i); else setOpen(null); };
  const leave = () => { setHover(null); closeTimer.current = setTimeout(() => setOpen((o) => (o === "primary" ? o : null)), 140); };

  return (
    <header className="topbar" ref={bar}>
      <Link className="topbar-logo" to="/" aria-label="CyborgPL, home">
        <img src={mark} alt="" width="30" height="30" />
      </Link>

      <nav className="topbar-nav" aria-label="Main" onPointerLeave={leave} onPointerEnter={() => clearTimeout(closeTimer.current)}>
        {NAV.map((item, i) => {
          const isOpen = open === i;
          const inner = (
            <>
              {last === i && <motion.span className="nav-hover" layoutId="nav-hover" transition={glide} initial={{ opacity: 0 }} animate={{ opacity: hover === null && open !== i ? 0 : 1 }} />}
              <span className="nav-label">{item.label}</span>
              {item.menu && <Chevron open={isOpen} />}
            </>
          );
          return (
            <div key={item.label} className="nav-item" ref={(el) => { items.current[i] = el; }} onPointerEnter={() => enter(i)}>
              {item.menu ? (
                <button className="nav-button" aria-expanded={isOpen} aria-haspopup="menu" onClick={() => (isOpen ? setOpen(null) : openMenu(i))} onFocus={() => { setHover(i); setLast(i); }} onBlur={() => setHover(null)}>{inner}</button>
              ) : (
                <Link className="nav-button" to={item.to!} onFocus={() => { setHover(i); setLast(i); }} onBlur={() => setHover(null)}>{inner}</Link>
              )}
            </div>
          );
        })}
        <AnimatePresence>
          {typeof open === "number" && NAV[open]!.menu && (
            <SharedMenu index={open} from={from} x={items.current[open]?.offsetLeft ?? 0} onPick={() => setOpen(null)} />
          )}
        </AnimatePresence>
      </nav>

      <div className="topbar-actions">
        <motion.div className="split" whileHover={{ scale: 1.03 }} whileTap={{ scale: 0.97 }} transition={glide}>
          <Link className="split-main" to={PRIMARY.to}>{PRIMARY.label}</Link>
          <button className="split-more" aria-label="Downloads" aria-expanded={open === "primary"} aria-haspopup="menu" onClick={() => setOpen(open === "primary" ? null : "primary")}>
            <Chevron open={open === "primary"} />
          </button>
        </motion.div>
        <AnimatePresence>{open === "primary" && <Menu links={PRIMARY.menu} onPick={() => setOpen(null)} align="right" />}</AnimatePresence>
        <button className="sheet-toggle" aria-label={sheet ? "Close the menu" : "Open the menu"} aria-expanded={sheet} onClick={() => setSheet(!sheet)}>
          <svg width="20" height="20" viewBox="0 0 20 20" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" aria-hidden="true">
            <motion.path d="M3 6.5 17 6.5" initial={false} animate={{ d: sheet ? "M5 5 15 15" : "M3 6.5 17 6.5" }} transition={glide} />
            <motion.path d="M3 13.5 17 13.5" initial={false} animate={{ d: sheet ? "M5 15 15 5" : "M3 13.5 17 13.5" }} transition={glide} />
          </svg>
        </button>
      </div>

      <AnimatePresence>
        {sheet && (
          <motion.div className="sheet" initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }} transition={{ duration: 0.2 }}>
            {NAV.map((item, i) => (
              <motion.div key={item.label} className="sheet-group" initial={{ opacity: 0, y: 12 }} animate={{ opacity: 1, y: 0 }} transition={{ delay: 0.04 * i, ...glide }}>
                {item.menu ? (
                  <>
                    <p className="sheet-heading">{item.label}</p>
                    {item.menu.map((l) => (l.to.startsWith("https://")
                      ? <a key={l.label} className="sheet-link" href={l.to} target="_blank" rel="noopener noreferrer">{l.label} ↗</a>
                      : <Link key={l.label} className="sheet-link" to={l.to}>{l.label}</Link>))}
                  </>
                ) : (
                  <Link className="sheet-link sheet-top" to={item.to!}>{item.label}</Link>
                )}
              </motion.div>
            ))}
          </motion.div>
        )}
      </AnimatePresence>
    </header>
  );
}
