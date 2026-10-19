import { AnimatePresence, motion } from "motion/react";
import { useEffect, useRef, useState } from "react";
import { Link, useLocation } from "react-router";
import mark from "../assets/mark-white.svg?url";
import { NAV, PRIMARY, SECONDARY, type NavLink } from "./nav";

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

export function Topbar() {
  // One highlight: it sits under the item last pointed at, slides to the next (layoutId), and
  // fades out, staying put, when the pointer leaves the bar.
  const [hover, setHover] = useState<number | null>(null);
  const [last, setLast] = useState<number | null>(null);
  const [open, setOpen] = useState<number | "primary" | null>(null);
  const [sheet, setSheet] = useState(false);
  const bar = useRef<HTMLElement>(null);
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
  const enter = (i: number) => { clearTimeout(closeTimer.current); setHover(i); setLast(i); if (NAV[i]!.menu) setOpen(i); else setOpen(null); };
  const leave = () => { setHover(null); closeTimer.current = setTimeout(() => setOpen((o) => (o === "primary" ? o : null)), 140); };

  return (
    <header className="topbar" ref={bar}>
      <Link className="topbar-logo" to="/" aria-label="CyborgPL, home">
        <img src={mark} alt="" width="30" height="30" />
      </Link>

      <nav className="topbar-nav" aria-label="Main" onPointerLeave={leave}>
        {NAV.map((item, i) => {
          const isOpen = open === i;
          const inner = (
            <>
              {last === i && <motion.span className="nav-hover" layoutId="nav-hover" transition={glide} initial={{ opacity: 0 }} animate={{ opacity: hover === null ? 0 : 1 }} />}
              <span className="nav-label">{item.label}</span>
              {item.menu && <Chevron open={isOpen} />}
            </>
          );
          return (
            <div key={item.label} className="nav-item" onPointerEnter={() => enter(i)}>
              {item.menu ? (
                <button className="nav-button" aria-expanded={isOpen} aria-haspopup="menu" onClick={() => setOpen(isOpen ? null : i)} onFocus={() => { setHover(i); setLast(i); }} onBlur={() => setHover(null)}>{inner}</button>
              ) : (
                <Link className="nav-button" to={item.to!} onFocus={() => { setHover(i); setLast(i); }} onBlur={() => setHover(null)}>{inner}</Link>
              )}
              <AnimatePresence>{isOpen && item.menu && <Menu links={item.menu} onPick={() => setOpen(null)} />}</AnimatePresence>
            </div>
          );
        })}
      </nav>

      <div className="topbar-actions">
        <motion.span className="pressable" whileHover={{ scale: 1.03 }} whileTap={{ scale: 0.97 }} transition={glide}>
          <Link className="button-dark" to={SECONDARY.to}>{SECONDARY.label}</Link>
        </motion.span>
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
                    {item.menu.map((l) => <Link key={l.label} className="sheet-link" to={l.to}>{l.label}</Link>)}
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
