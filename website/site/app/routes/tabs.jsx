import { Link, Outlet, useLocation } from "react-router";
import { motion } from "motion/react";
import Icon from "../components/Icon.jsx";

const TABS = [
  { to: "/", icon: "home", label: "Home", match: (p) => p === "/" },
  { to: "/learn", icon: "book", label: "Learn", match: (p) => p.startsWith("/learn") },
  { to: "/examples", icon: "code", label: "Examples", match: (p) => p.startsWith("/examples") },
  { to: "/more", icon: "more", label: "More", match: (p) => ["/more", "/download", "/licence"].some((x) => p.startsWith(x)) },
];

export default function Tabs() {
  const { pathname } = useLocation();
  // The top-level tab decides the page transition; opening an example (a sheet) doesn't.
  const section = "/" + (pathname.split("/")[1] ?? "");
  return (
    <>
      <main className="page" key={section}>
        <Outlet />
      </main>
      <nav className="tabbar" aria-label="Sections">
        {TABS.map((t) => {
          const on = t.match(pathname);
          return (
            <Link key={t.to} to={t.to} prefetch="render" className={"tab" + (on ? " on" : "")} aria-current={on ? "page" : undefined} aria-label={t.label}>
              {on && <motion.span layoutId="tab-pill" className="tab-pill" transition={{ type: "spring", stiffness: 900, damping: 55, mass: 0.6 }} />}
              <span className="tab-inner">
                <Icon name={t.icon} size={22} />
                {on && <motion.b initial={{ opacity: 0 }} animate={{ opacity: 1 }} transition={{ duration: 0.1 }}>{t.label}</motion.b>}
              </span>
            </Link>
          );
        })}
      </nav>
    </>
  );
}
