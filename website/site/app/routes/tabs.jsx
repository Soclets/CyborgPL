import { useLayoutEffect, useRef } from "react";
import { Link, Outlet, useLocation, useNavigate } from "react-router";
import Icon from "../components/Icon.jsx";

const TABS = [
  { to: "/", icon: "home", label: "Home", match: (p) => p === "/" },
  { to: "/learn", icon: "book", label: "Learn", match: (p) => p.startsWith("/learn") },
  { to: "/examples", icon: "code", label: "Examples", match: (p) => p.startsWith("/examples") },
  { to: "/more", icon: "more", label: "More", match: (p) => ["/more", "/download", "/licence"].some((x) => p.startsWith(x)) },
];

// Tapping the tab you're on: the first tap goes to the top, the next clears the search.
function retap() {
  if (window.scrollY > 0) window.scrollTo({ top: 0, behavior: "smooth" });
  else window.dispatchEvent(new Event("cyborgpl:retap"));
}

export default function Tabs() {
  const { pathname } = useLocation();
  const navigate = useNavigate();
  // The top-level tab decides the page transition; opening an example (a sheet) doesn't.
  const section = "/" + (pathname.split("/")[1] ?? "");

  // The pill slides from the old tab to the new one (measured before and after, then
  // animated by the browser), so the tab bar needs no animation library.
  const pill = useRef(null);
  const was = useRef(null);
  useLayoutEffect(() => {
    const el = pill.current;
    if (!el) return;
    const now = el.getBoundingClientRect();
    const from = was.current;
    was.current = now;
    if (!from || (from.left === now.left && from.width === now.width) || matchMedia("(prefers-reduced-motion: reduce)").matches) return;
    el.animate(
      [{ transform: `translateX(${from.left - now.left}px)`, width: `${from.width}px` }, { transform: "none", width: `${now.width}px` }],
      { duration: 260, easing: "cubic-bezier(.3, 1.3, .5, 1)" },
    );
  }, [section]);

  // A finger acts on lift, not on the click that follows. iOS drops that click when the
  // tap lands while the page is still coasting from a scroll, and that is when people tap tabs.
  const tapped = useRef(0);
  const onPointerUp = (t, e) => {
    if (e.pointerType === "mouse" || e.button !== 0) return;
    if (!e.currentTarget.contains(document.elementFromPoint(e.clientX, e.clientY))) return; // slid off
    tapped.current = Date.now();
    if (pathname === t.to) retap();
    else navigate(t.to);
  };
  const onClick = (t, e) => {
    if (Date.now() - tapped.current < 700) e.preventDefault(); // already handled on lift
    else if (pathname === t.to) { e.preventDefault(); retap(); }
  };

  return (
    <>
      <main className="page" key={section}>
        <Outlet />
      </main>
      <nav className="tabbar" aria-label="Sections">
        {TABS.map((t) => {
          const on = t.match(pathname);
          return (
            <Link key={t.to} to={t.to} prefetch="render" className={"tab" + (on ? " on" : "")} aria-current={on ? "page" : undefined} aria-label={t.label}
              onPointerUp={(e) => onPointerUp(t, e)} onClick={(e) => onClick(t, e)} draggable={false}>
              {on && <span ref={pill} className="tab-pill" />}
              <span className="tab-inner">
                <Icon name={t.icon} size={22} />
                {on && <b>{t.label}</b>}
              </span>
            </Link>
          );
        })}
      </nav>
    </>
  );
}
