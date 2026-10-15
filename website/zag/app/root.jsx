import { useEffect, useState } from "react";
import { Links, Meta, Outlet, Scripts, ScrollRestoration } from "react-router";
import { Backdrop, Pill } from "./sky.jsx";
import "./app.css";
// Fredoka, cut to the characters the site shows, its width fixed and its weights 300 to 700 kept
// (owner, 2026-10-15: "optimize the website"). Built under a hashed name, so a new cut is never
// stuck behind the year-long cache.
import fredoka from "./fonts/fredoka.woff2?url";
import logo from "./logo.svg?url";

export const links = () => [
  { rel: "icon", href: "/favicon.svg", type: "image/svg+xml" },
  { rel: "icon", href: "/favicon.ico", sizes: "32x32" },
  { rel: "apple-touch-icon", href: "/apple-touch-icon.png" },
  { rel: "preload", href: fredoka, as: "font", type: "font/woff2", crossOrigin: "anonymous" },
];

export function Layout({ children }) {
  return (
    <html lang="en">
      <head>
        <meta charSet="utf-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover" />
        <meta name="theme-color" content="#1E64C8" />
        {/* The loading screen shows only when scripts run, since only they can take it away. */}
        <script dangerouslySetInnerHTML={{ __html: 'document.documentElement.classList.add("loading")' }} />
        <Meta />
        <Links />
      </head>
      <body>
        <Loading />
        {children}
        <ScrollRestoration />
        <Scripts />
      </body>
    </html>
  );
}

// The loading screen (owner, 2026-10-15: "make it a real loading screen"): on every full load it
// fetches the whole site, every page's code with its data, and the font, and its bar fills as each
// arrives. Then it fades and the page starts, so no click afterwards waits on the network. Moving
// between pages never shows it again. If something fails it counts as done; after 10 seconds it
// gives up and lets the page show anyway.
function Loading() {
  const [part, setPart] = useState(0);
  const [gone, setGone] = useState(false);
  useEffect(() => {
    const root = document.documentElement;
    if (!root.classList.contains("loading")) { setGone(true); return; }
    const routes = Object.values(window.__reactRouterManifest?.routes ?? {});
    const jobs = [
      ...routes.map((r) => import(/* @vite-ignore */ r.module)),
      document.fonts.load("400 1em Fredoka").then(() => document.fonts.ready),
      document.readyState === "complete" ? Promise.resolve() : new Promise((go) => addEventListener("load", go, { once: true })),
    ];
    let done = 0, over = false;
    const finish = () => {
      if (over) return;
      over = true;
      setPart(1);
      // The bar reaches the end, then the screen fades (app.css) and the page's own motion starts.
      setTimeout(() => {
        root.classList.replace("loading", "loaded");
        setTimeout(() => setGone(true), 600);
      }, 180);
    };
    for (const job of jobs) job.catch(() => {}).then(() => { done += 1; setPart(done / jobs.length); if (done === jobs.length) finish(); });
    const giveUp = setTimeout(finish, 10000);
    return () => clearTimeout(giveUp);
  }, []);
  if (gone) return null;
  return (
    <div className="loader" role="progressbar" aria-label="Loading Zag" aria-valuemin={0} aria-valuemax={100} aria-valuenow={Math.round(part * 100)}>
      <img className="loader-logo" src={logo} alt="" width="196" height="147" />
      <div className="loader-bar"><span style={{ transform: `scaleX(${part})` }} /></div>
    </div>
  );
}

// The sky and the pill belong to the whole site, not to a page: pages change on top of them
// without a reload, so the clouds drift on across a click and the pill never redraws (owner:
// "The transition of clicking a pill button is still bad, because the background doesn't align").
export default function App() {
  return (
    <>
      <Backdrop />
      <Pill />
      <Outlet />
    </>
  );
}
