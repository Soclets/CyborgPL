import { useEffect, useId, useLayoutEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { AnimatePresence, motion, useIsPresent, useReducedMotion } from "motion/react";
import Icon from "./Icon.jsx";
import LanguageLogo from "./LanguageLogo.jsx";
import { COMPARISON_LANGUAGES } from "../lib/comparison.js";

function Options({ id, side, value, other, active, position, menuRef, onActive, onChoose }) {
  const present = useIsPresent();
  const reduced = useReducedMotion();
  return (
    <motion.div ref={menuRef} id={id} role="listbox" aria-label={`${side} languages`}
      aria-hidden={!present} inert={!present ? true : undefined} className="language-menu"
      style={{ ...position, pointerEvents: present ? "auto" : "none" }}
      initial={{ opacity: 0, y: reduced ? 0 : -4 }} animate={{ opacity: 1, y: 0 }} exit={{ opacity: 0, y: reduced ? 0 : -3 }}
      transition={{ duration: reduced ? 0 : 0.12 }}>
      {COMPARISON_LANGUAGES.map((l) => (
        <button key={l.id} id={`${id}-${l.id}`} type="button" role="option" tabIndex={-1}
          aria-selected={active === l.id} aria-disabled={l.id === other} disabled={l.id === other}
          className={`language-option${active === l.id ? " is-active" : ""}${value === l.id ? " is-current" : ""}`}
          onMouseDown={(e) => e.preventDefault()}
          onPointerMove={(e) => { if (e.pointerType !== "touch" && l.id !== other) onActive(l.id); }}
          onClick={() => onChoose(l.id)}>
          <LanguageLogo id={l.id} />
          <span><b>{l.name}</b><small>{l.version ?? l.status}{l.id === other ? " · Other side" : ""}</small></span>
          {value === l.id && <Icon name="check" size={18} />}
        </button>
      ))}
    </motion.div>
  );
}

// Select-only combobox: focus stays on the trigger while aria-activedescendant
// identifies the explored option. Enter/Space/Tab commit; Escape cancels.
export default function LanguagePicker({ side, value, other, onChange }) {
  const id = useId();
  const trigger = useRef(null), menu = useRef(null);
  const typed = useRef({ text: "", at: 0 });
  const [open, setOpen] = useState(false);
  const [active, setActive] = useState(value);
  const [position, setPosition] = useState({ top: 0, left: 0, width: 264, maxHeight: 260 });
  const selected = COMPARISON_LANGUAGES.find((l) => l.id === value);
  const enabled = COMPARISON_LANGUAGES.filter((l) => l.id !== other);

  const show = (next = value) => { setActive(next); setOpen(true); };
  const choose = (next = active, refocus = true) => {
    if (next !== other) onChange(next);
    setOpen(false);
    if (refocus) trigger.current.focus({ preventScroll: true });
  };

  useLayoutEffect(() => {
    if (!open) return;
    const place = () => {
      const r = trigger.current.getBoundingClientRect();
      const viewport = window.visualViewport;
      const x = viewport?.offsetLeft ?? 0, y = viewport?.offsetTop ?? 0;
      const width = viewport?.width ?? innerWidth, height = viewport?.height ?? innerHeight;
      const topbar = document.querySelector(".topbar")?.getBoundingClientRect();
      const tabbar = document.querySelector(".tabbar")?.getBoundingClientRect();
      const top = Math.max(y + 12, topbar?.width ? topbar.bottom + 8 : y + 12);
      const bottom = Math.min(y + height - 12, tabbar?.width ? tabbar.top - 8 : y + height - 12);
      if (r.bottom < top || r.top > bottom) return setOpen(false);
      const below = Math.max(0, bottom - r.bottom - 8), above = Math.max(0, r.top - top - 8);
      const natural = (menu.current?.scrollHeight ?? 250) + 2;
      const up = below < natural && above > below;
      const maxHeight = Math.min(natural, up ? above : below);
      const menuWidth = Math.min(Math.max(r.width, 264), width - 24);
      const left = Math.min(Math.max(side === "Left" ? r.left : r.right - menuWidth, x + 12), x + width - menuWidth - 12);
      setPosition({ top: up ? r.top - maxHeight - 8 : r.bottom + 8, left, width: menuWidth, maxHeight });
    };
    place();
    addEventListener("resize", place);
    addEventListener("scroll", place, true);
    window.visualViewport?.addEventListener("resize", place);
    window.visualViewport?.addEventListener("scroll", place);
    return () => {
      removeEventListener("resize", place);
      removeEventListener("scroll", place, true);
      window.visualViewport?.removeEventListener("resize", place);
      window.visualViewport?.removeEventListener("scroll", place);
    };
  }, [open, side]);

  useLayoutEffect(() => {
    if (open) document.getElementById(`${id}-${active}`)?.scrollIntoView({ block: "nearest" });
  }, [open, active, id]);

  useEffect(() => {
    if (!open) return;
    const outside = (e) => { if (!trigger.current.contains(e.target) && !menu.current?.contains(e.target)) setOpen(false); };
    document.addEventListener("pointerdown", outside);
    return () => document.removeEventListener("pointerdown", outside);
  }, [open]);

  const onKey = (e) => {
    const index = enabled.findIndex((l) => l.id === active);
    if (e.key === "Escape" && open) { e.preventDefault(); e.stopPropagation(); setOpen(false); }
    else if (e.key === "Tab") { if (open) choose(active, false); }
    else if (e.key === "Enter" || e.key === " ") { e.preventDefault(); open ? choose() : show(); }
    else if (e.key === "ArrowDown" || e.key === "ArrowUp") {
      e.preventDefault();
      if (e.altKey && e.key === "ArrowUp" && !open) return;
      if (e.altKey && e.key === "ArrowUp" && open) return choose();
      if (!open || e.altKey) return show(e.key === "ArrowUp" && !e.altKey ? enabled[0].id : value);
      setActive(enabled[Math.max(0, Math.min(enabled.length - 1, index + (e.key === "ArrowDown" ? 1 : -1)))].id);
    } else if (["Home", "End", "PageUp", "PageDown"].includes(e.key)) {
      e.preventDefault();
      show(enabled[e.key === "Home" || e.key === "PageUp" ? 0 : enabled.length - 1].id);
    } else if (e.key.length === 1 && !e.ctrlKey && !e.metaKey && !e.altKey) {
      e.preventDefault();
      const now = Date.now(), key = e.key.toLowerCase();
      const text = now - typed.current.at < 700 && typed.current.text !== key ? typed.current.text + key : key;
      typed.current = { text, at: now };
      const match = enabled.find((l) => l.name.toLowerCase().startsWith(text));
      if (match) show(match.id);
    }
  };

  return (
    <div className="comparison-picker">
      <span id={`${id}-label`}>{side} side</span>
      <button ref={trigger} type="button" role="combobox" aria-label={`${side} comparison language`}
        aria-haspopup="listbox" aria-expanded={open} aria-controls={id}
        aria-activedescendant={open ? `${id}-${active}` : undefined}
        className={`comparison-trigger${open ? " is-open" : ""}`} onKeyDown={onKey}
        onClick={() => open ? setOpen(false) : show()}
        onBlur={(e) => { if (!menu.current?.contains(e.relatedTarget)) setOpen(false); }}>
        <LanguageLogo id={selected.id} className="picker-logo" />
        <span><b>{selected.name}</b><small>{selected.version?.replace(/^CPython /, "") ?? "Unreleased"}</small></span>
        <Icon name="chev" size={16} />
      </button>
      {typeof document !== "undefined" && createPortal(
        <AnimatePresence>{open && <Options key={id} id={id} side={side} value={value} other={other} active={active}
          position={position} menuRef={menu} onActive={setActive} onChoose={choose} />}</AnimatePresence>, document.body)}
    </div>
  );
}
