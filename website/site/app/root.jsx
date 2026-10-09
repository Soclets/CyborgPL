import { useEffect } from "react";
import { wordmarkSVG } from "./lib/wordmark.js";
import { ICON } from "./lib/icon-hash.js";
import { Links, Meta, Outlet, Scripts, ScrollRestoration } from "react-router";
import "./app.css";

export const links = () => [
  { rel: "manifest", href: "/manifest.webmanifest" },
  // The tab icon (brand/nerve/icon.svg, the Nerve icon on a black tile): ICO and PNG for Safari and
  // older browsers, SVG for the rest. The file names carry a hash of the drawing (scripts/icons.mjs),
  // so a changed icon is an address no browser has cached, Safari's icon cache included.
  { rel: "icon", href: `/icons/tab-${ICON}.ico`, sizes: "48x48" },
  { rel: "icon", href: `/icons/tab-${ICON}-32.png`, type: "image/png", sizes: "32x32" },
  { rel: "icon", href: `/icons/tab-${ICON}-96.png`, type: "image/png", sizes: "96x96" },
  { rel: "icon", href: `/icons/tab-${ICON}.svg`, type: "image/svg+xml" },
  { rel: "apple-touch-icon", href: `/icons/touch-${ICON}.png` },
];

// The loading screen (owner, 2026-09-28: "a real loading screen"). It is in every pre-rendered
// page, so it is there from the first paint: the logo, large, and a spinning indicator (owner,
// 2026-10-09: the progress bar replaced by a spinner). It stays until loading really ends: every
// file the page downloads (its scripts and styles, as the browser reports them finished) and then
// the app itself starting. It never shows without JavaScript, stays up at least 1 s even when
// loading is quicker (owner), gives up after 10 s rather than stick, and holds no text, so a
// search engine has nothing of it to quote.
const MARK = `<div class="boot-mark logo">${wordmarkSVG("wordmark-boot", 48)}</div>`;
const BOOT_HTML = `<div class="boot-in">${MARK}<span class="boot-spin logo"></span></div>`;
const BOOT_JS = `(function(){
var b=document.getElementById("boot");if(!b)return;var shown=performance.now();
var over=false;
function done(){if(over)return;over=true;b.removeAttribute("aria-busy");setTimeout(function(){b.setAttribute("data-done","")},Math.max(0,1000-(performance.now()-shown)))}
window.__nerveStarted=done;setTimeout(done,10000);
})();`;

export function Layout({ children }) {
  return (
    <html lang="en" suppressHydrationWarning>
      <head>
        <meta charSet="utf-8" />
        {/* A computer (a mouse, not a finger) gets the desktop layout, in Solarized Light or Dark
            (owner, 2026-09-28: temporary). Decided before anything paints, so nothing flashes. */}
        <script dangerouslySetInnerHTML={{ __html: `(function(){var h=document.documentElement;if(matchMedia("(hover: hover) and (pointer: fine)").matches)h.className="desktop";try{var t=localStorage.getItem("nerve.theme");if(t==="light"||t==="dark")h.dataset.theme=t}catch(e){}})();` }} />
        {/* viewport-fit=cover: draw under the notch and home bar; the CSS keeps content clear of them */}
        <meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover" />
        <meta name="theme-color" content="#fdf6e3" />
        <meta name="color-scheme" content="light" />
        {/* Added to the home screen, it opens full-screen like an app */}
        <meta name="apple-mobile-web-app-capable" content="yes" />
        <meta name="mobile-web-app-capable" content="yes" />
        <meta name="apple-mobile-web-app-title" content="Nerve" />
        <meta name="apple-mobile-web-app-status-bar-style" content="default" />
        <Meta />
        <Links />
      </head>
      <body>
        <div id="boot" role="status" aria-label="Loading" aria-busy="true" suppressHydrationWarning dangerouslySetInnerHTML={{ __html: BOOT_HTML }} />
        <noscript><style>{"#boot{display:none}"}</style></noscript>
        <script dangerouslySetInnerHTML={{ __html: BOOT_JS }} />
        {children}
        <ScrollRestoration />
        <Scripts />
      </body>
    </html>
  );
}

// The app is running: the last step of the loading screen.
const useStarted = () => useEffect(() => { window.__nerveStarted?.(); }, []);

export default function App() {
  useStarted();
  return <Outlet />;
}

export function ErrorBoundary() {
  useStarted();
  return (
    <main className="page">
      <h1 className="h2big">Something broke.</h1>
      <p className="lede">This screen failed to load. <a href="/">Back to the start</a>.</p>
    </main>
  );
}
