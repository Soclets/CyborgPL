import { useLayoutEffect, useRef } from "react";
import { Link, Outlet, useLocation, useNavigate, useNavigationType } from "react-router";
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

  // One pill for the whole bar, moved by a CSS transition: a tap during a slide carries on
  // from wherever the pill is, instead of restarting. Back and forward don't slide it —
  // Safari's own back animation is already playing then, and two would show.
  const nav = useRef(null);
  const pill = useRef(null);
  const placed = useRef(false);
  const navType = useNavigationType();
  useLayoutEffect(() => {
    const place = (slide) => {
      const on = nav.current.querySelector(".tab.on"), p = pill.current;
      if (!on) return void (p.style.opacity = "0");
      p.style.transition = slide ? "" : "none";
      p.style.transform = `translateX(${on.offsetLeft + 4}px)`;
      p.style.width = `${on.offsetWidth - 8}px`;
      p.style.opacity = "1";
      if (!slide) p.getBoundingClientRect(); // settle the jump before transitions return
    };
    place(placed.current && navType !== "POP");
    placed.current = true;
    nav.current.classList.add("ready");
    const onResize = () => place(false);
    addEventListener("resize", onResize);
    return () => removeEventListener("resize", onResize);
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
      <nav className="tabbar" aria-label="Sections" ref={nav}>
        <span className="tab-pill" ref={pill} />
        {TABS.map((t) => {
          const on = t.match(pathname);
          return (
            <Link key={t.to} to={t.to} prefetch="render" className={"tab" + (on ? " on" : "")} aria-current={on ? "page" : undefined} aria-label={t.label}
              onPointerUp={(e) => onPointerUp(t, e)} onClick={(e) => onClick(t, e)} draggable={false}>
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
