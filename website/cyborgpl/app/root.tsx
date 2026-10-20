import type { ReactNode } from "react";
import { Links, Meta, Outlet, Scripts, ScrollRestoration } from "react-router";
import { LoadingScreen } from "./loading/loading-screen";
import { Topbar } from "./components/topbar";
import "./app.css";

export function Layout({ children }: { children: ReactNode }) {
  return (
    // The early script adds .loading to <html> before React starts, on purpose.
    <html lang="en" suppressHydrationWarning>
      <head>
        <meta charSet="utf-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover" />
        {/* The loading screen shows only when scripts run, since only they can take it away. */}
        <script dangerouslySetInnerHTML={{ __html: 'document.documentElement.classList.add("loading")' }} />
        <Meta />
        <Links />
      </head>
      <body>
        {/* Shown only with JavaScript off (owner, 2026-10-20: "We could display a "We need JS" message"):
            the sections below the hero appear as scripts animate them in. */}
        <noscript>
          <div className="nojs" role="status"><strong>We need JavaScript.</strong> Turn it on to see the whole page.</div>
        </noscript>
        <LoadingScreen />
        {children}
        <ScrollRestoration />
        <Scripts />
      </body>
    </html>
  );
}

// The topbar belongs to the whole site, so it stays put while pages change under it.
export default function App() {
  return (
    <>
      <Topbar />
      <Outlet />
    </>
  );
}
