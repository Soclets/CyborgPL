import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { Link, Outlet, useLocation, useNavigate, useNavigation, useNavigationType } from "react-router";
import Icon from "../components/Icon.jsx";
import Mark from "../components/Mark.jsx";
import { REPO_URL } from "../lib/content.js";

const TABS = [
  { to: "/", icon: "home", label: "Home", match: (p) => p === "/" },
  { to: "/learn", icon: "book", label: "Learn", match: (p) => p.startsWith("/learn") || p.startsWith("/good-practice") },
  { to: "/examples", icon: "code", label: "Examples", match: (p) => p.startsWith("/examples") },
  { to: "/more", icon: "more", label: "More", match: (p) => ["/more", "/download", "/licence"].some((x) => p.startsWith(x)) },
];

// The desktop site's top bar (a computer only; phones keep the tab bar). Download and
// Licence sit in it directly, since a wide screen has room for them.
const LINKS = [
  { to: "/", label: "Home", match: (p) => p === "/" },
  { to: "/learn", label: "Learn", match: (p) => p.startsWith("/learn") || p.startsWith("/good-practice") },
  { to: "/examples", label: "Examples", match: (p) => p.startsWith("/examples") },
  { to: "/download", label: "Download", match: (p) => p.startsWith("/download") },
  { to: "/licence", label: "Licence", match: (p) => p.startsWith("/licence") },
  { to: "/commits", label: "Commits", match: (p) => p.startsWith("/commits") },
  { to: "/more", label: "More", match: (p) => p.startsWith("/more") },     // and RRx2, under Related
];

// Light, dark, or whatever the computer is set to. Kept on this computer; root.jsx applies
// it before the first paint.
const THEMES = [
  { key: "system", icon: "auto", label: "Theme: follows the system" },
  { key: "light", icon: "sun", label: "Theme: Solarized Light" },
  { key: "dark", icon: "moon", label: "Theme: Solarized Dark" },
];
function ThemeSwitch() {
  const [theme, setTheme] = useState("system");
  useEffect(() => setTheme(document.documentElement.dataset.theme ?? "system"), []);
  const next = () => {
    const t = THEMES[(THEMES.findIndex((x) => x.key === theme) + 1) % THEMES.length].key;
    setTheme(t);
    if (t === "system") delete document.documentElement.dataset.theme;
    else document.documentElement.dataset.theme = t;
    try { t === "system" ? localStorage.removeItem("nerve.theme") : localStorage.setItem("nerve.theme", t); } catch {}
  };
  const cur = THEMES.find((x) => x.key === theme);
  return <button className="theme-switch" onClick={next} aria-label={cur.label} title={cur.label}><Icon name={cur.icon} size={19} /></button>;
}

function Topbar({ pathname }) {
  const navigation = useNavigation();
  const target = navigation.location?.pathname ?? pathname;
  const active = LINKS.findIndex((l) => l.match(target));
  const nav = useRef(null);
  const pill = useRef(null);
  const placed = useRef(false);

  // One highlight moves as navigation starts, without waiting for the page's
  // data. Repeated clicks retarget the same CSS transition from its current spot.
  useLayoutEffect(() => {
    const place = (slide) => {
      const on = nav.current.querySelector("a.on"), p = pill.current;
      if (!on) return void (p.style.opacity = "0");
      const button = on.getBoundingClientRect(), bar = nav.current.getBoundingClientRect();
      p.style.transition = slide ? "" : "none";
      p.style.transform = `translateX(${button.left - bar.left}px)`;
      p.style.width = `${button.width}px`;
      p.style.opacity = "1";
      if (!slide) p.getBoundingClientRect();
    };
    place(placed.current);
    placed.current = true;
    nav.current.classList.add("ready");
    const onResize = () => place(false);
    addEventListener("resize", onResize);
    return () => removeEventListener("resize", onResize);
  }, [active]);

  return (
    <header className="topbar">
      <div className="topbar-in">
        <Link to="/" className="brand" prefetch="intent" aria-label="Nerve, home"><Mark height={18} /></Link>
        <nav aria-label="Site" ref={nav}>
          <span className="topbar-pill" ref={pill} aria-hidden="true" />
          {LINKS.map((l, i) => (
            <Link key={l.to} to={l.to} prefetch="intent" className={i === active ? "on" : ""} aria-current={l.match(pathname) ? "page" : undefined}>{l.label}</Link>
          ))}
        </nav>
        <a className="topbar-icon" href={REPO_URL} target="_blank" rel="noopener noreferrer" aria-label="Source on GitHub" title="Source on GitHub"><Icon name="github" size={19} stroke={1.9} /></a>
        <ThemeSwitch />
      </div>
    </header>
  );
}

// Tapping the tab you're on: the first tap goes to the top, the next clears the search.
function retap() {
  if (window.scrollY > 0) window.scrollTo({ top: 0, behavior: "smooth" });
  else window.dispatchEvent(new Event("nerve:retap"));
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
      <Topbar pathname={pathname} />
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
