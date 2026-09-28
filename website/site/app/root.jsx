import { Links, Meta, Outlet, Scripts, ScrollRestoration } from "react-router";
import "./app.css";

export const links = () => [
  { rel: "manifest", href: "/manifest.webmanifest" },
  { rel: "icon", href: "/favicon.svg", type: "image/svg+xml" },
  { rel: "apple-touch-icon", href: "/icons/apple-touch-icon.png" },
];

export function Layout({ children }) {
  return (
    <html lang="en" suppressHydrationWarning>
      <head>
        <meta charSet="utf-8" />
        {/* A computer (a mouse, not a finger) gets the desktop layout, in Solarized Light or Dark
            (owner, 2026-09-28: temporary). Decided before anything paints, so nothing flashes. */}
        <script dangerouslySetInnerHTML={{ __html: `(function(){var h=document.documentElement;if(matchMedia("(hover: hover) and (pointer: fine)").matches)h.className="desktop";try{var t=localStorage.getItem("cyborgpl.theme");if(t==="light"||t==="dark")h.dataset.theme=t}catch(e){}})();` }} />
        {/* viewport-fit=cover: draw under the notch and home bar; the CSS keeps content clear of them */}
        <meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover" />
        <meta name="theme-color" content="#fdf6e3" />
        <meta name="color-scheme" content="light" />
        {/* Added to the home screen, it opens full-screen like an app */}
        <meta name="apple-mobile-web-app-capable" content="yes" />
        <meta name="mobile-web-app-capable" content="yes" />
        <meta name="apple-mobile-web-app-title" content="CyborgPL" />
        <meta name="apple-mobile-web-app-status-bar-style" content="default" />
        <Meta />
        <Links />
      </head>
      <body>
        {children}
        <ScrollRestoration />
        <Scripts />
      </body>
    </html>
  );
}

export default function App() {
  return <Outlet />;
}

export function ErrorBoundary() {
  return (
    <main className="page">
      <h1 className="h2big">Something broke.</h1>
      <p className="lede">This screen failed to load. <a href="/">Back to the start</a>.</p>
    </main>
  );
}
