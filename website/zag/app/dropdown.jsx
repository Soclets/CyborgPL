import { useEffect, useId, useRef, useState } from "react";

// The site's own dropdown (owner, 2026-10-15: "Make it use our own dropdown"): a glass pill that
// opens a frosted list. Arrow keys move, Enter or Space picks, Escape closes, a letter jumps to the
// next name starting with it; a click elsewhere closes it. Marked up as a listbox for screen readers.
export default function Dropdown({ label, options, value, onChange }) {
  const [open, setOpen] = useState(false);
  const [active, setActiveState] = useState(value);
  const activeRef = useRef(value);
  // The highlighted option, kept in a ref too, so keys pressed in quick succession each see the last.
  const setActive = (v) => { const next = typeof v === "function" ? v(activeRef.current) : v; activeRef.current = next; setActiveState(next); };
  const box = useRef(null), button = useRef(null), list = useRef(null);
  const id = useId();
  const usable = options.filter((o) => !o.disabled);
  const current = options.find((o) => o.value === value);

  useEffect(() => {
    if (!open) return;
    setActive(value);
    list.current?.focus({ preventScroll: true });
    const away = (e) => { if (!box.current?.contains(e.target)) setOpen(false); };
    addEventListener("pointerdown", away);
    return () => removeEventListener("pointerdown", away);
  }, [open, value]);

  const pick = (v) => { onChange(v); setOpen(false); button.current?.focus({ preventScroll: true }); };
  const move = (step) => setActive((a) => {
    const i = usable.findIndex((o) => o.value === a);
    return usable[(i + step + usable.length) % usable.length].value;
  });
  const onListKey = (e) => {
    const keys = { ArrowDown: () => move(1), ArrowUp: () => move(-1), Home: () => setActive(usable[0].value), End: () => setActive(usable.at(-1).value),
      Enter: () => pick(activeRef.current), " ": () => pick(activeRef.current), Escape: () => { setOpen(false); button.current?.focus({ preventScroll: true }); }, Tab: () => setOpen(false) };
    if (keys[e.key]) { if (e.key !== "Tab") e.preventDefault(); e.stopPropagation(); keys[e.key](); return; }
    if (e.key.length === 1 && /\S/.test(e.key)) {
      e.stopPropagation();
      setActive((a) => {
        const i = usable.findIndex((o) => o.value === a);
        const next = [...usable.slice(i + 1), ...usable.slice(0, i + 1)].find((o) => o.label.toLowerCase().startsWith(e.key.toLowerCase()));
        return next ? next.value : a;
      });
    }
  };
  const onButtonKey = (e) => {
    if (["ArrowDown", "ArrowUp", "Enter", " "].includes(e.key)) { e.preventDefault(); e.stopPropagation(); setOpen(true); }
  };

  return (
    <div className={`dropdown${open ? " open" : ""}`} ref={box}>
      <button type="button" ref={button} className="dropdown-button" aria-haspopup="listbox" aria-expanded={open} aria-controls={`${id}-list`}
        aria-label={`${label}: ${current?.label}`} onClick={() => setOpen((o) => !o)} onKeyDown={onButtonKey}>
        <span>{current?.label}</span>
        <svg className="dropdown-chevron" viewBox="0 0 12 12" width="12" height="12" aria-hidden="true"><path d="M2 4.5 6 8.5 10 4.5" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" /></svg>
      </button>
      <ul id={`${id}-list`} ref={list} className="dropdown-list" role="listbox" tabIndex={-1} aria-label={label} hidden={!open}
        aria-activedescendant={open ? `${id}-${active}` : undefined} onKeyDown={onListKey}>
        {options.map((o) => (
          <li key={o.value} id={`${id}-${o.value}`} role="option" aria-selected={o.value === value} aria-disabled={o.disabled || undefined}
            className={`dropdown-option${o.value === active ? " active" : ""}`}
            onPointerEnter={() => !o.disabled && setActive(o.value)} onClick={() => !o.disabled && pick(o.value)}>
            <span className="dropdown-check" aria-hidden="true">{o.value === value ? "✓" : ""}</span>
            <span className="dropdown-name">{o.label}</span>
            {o.note && <small>{o.note}</small>}
          </li>
        ))}
      </ul>
    </div>
  );
}
