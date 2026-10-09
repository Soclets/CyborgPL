import { useEffect } from "react";
import { wordmarkSVG } from "./lib/wordmark.js";
import { Links, Meta, Outlet, Scripts, ScrollRestoration } from "react-router";
import "./app.css";

export const links = () => [
  { rel: "manifest", href: "/manifest.webmanifest" },
  // The tab icon (brand/nerve/icon.svg, the R from the wordmark): ICO and PNG for Safari and older browsers, SVG for the rest.
  // ?v=3: a new address, so browsers that cached "no icon" (Safari) fetch these afresh.
  { rel: "icon", href: "/favicon.ico?v=3", sizes: "48x48" },
  { rel: "icon", href: "/icons/favicon-32.png?v=3", type: "image/png", sizes: "32x32" },
  { rel: "icon", href: "/icons/favicon-96.png?v=3", type: "image/png", sizes: "96x96" },
  { rel: "icon", href: "/favicon.svg?v=3", type: "image/svg+xml" },
  { rel: "apple-touch-icon", href: "/icons/apple-touch-icon.png?v=3" },
];

// The loading screen (owner, 2026-09-28: "a real loading screen"). It is in every
// pre-rendered page, so it is there from the first paint, and its bar is real: each step is
// a file the page actually downloads (its scripts and styles, counted as the browser
// reports them finished), and the last step is the app itself starting. It never shows
// without JavaScript, stays up at least 1 s even when loading is quicker (owner; the bar
// still reaches 100% when loading really ends), gives up after 10 s rather than stick, and holds no text, so a search
// engine has nothing of it to quote (the percentage is drawn by CSS from data-pct).
const MARK = wordmarkSVG("wordmark-boot", 30);
const BOOT_HTML = `<div class="boot-in">${MARK}<div class="boot-bar"><i></i></div><span class="boot-pct" data-pct="0%"></span></div>`;
const BOOT_JS = `(function(){
var b=document.getElementById("boot");if(!b)return;var shown=performance.now();
var pct=b.querySelector(".boot-pct");
var want=[].map.call(document.querySelectorAll('link[rel="modulepreload"],link[rel="stylesheet"]'),function(l){return l.href});
var total=want.length+1,seen={},n=0,over=false;
function show(p){var v=Math.round(p*100);b.style.setProperty("--p",p);pct.setAttribute("data-pct",v+"%");b.setAttribute("aria-valuenow",v)}
function got(u){if(over||seen[u]||want.indexOf(u)<0)return;seen[u]=1;n++;show(n/total)}
try{performance.getEntriesByType("resource").forEach(function(e){got(e.name)});new PerformanceObserver(function(l){l.getEntries().forEach(function(e){got(e.name)})}).observe({type:"resource",buffered:true})}catch(e){}
show(n/total);
function done(){if(over)return;over=true;show(1);setTimeout(function(){b.setAttribute("data-done","")},Math.max(0,1000-(performance.now()-shown)))}
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
        <div id="boot" role="progressbar" aria-label="Loading" aria-valuemin={0} aria-valuemax={100} suppressHydrationWarning dangerouslySetInnerHTML={{ __html: BOOT_HTML }} />
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
