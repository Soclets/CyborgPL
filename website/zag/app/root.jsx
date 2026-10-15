import { Links, Meta, Outlet, Scripts, ScrollRestoration } from "react-router";
import { Backdrop, Pill } from "./sky.jsx";
import "./app.css";

export const links = () => [
  { rel: "icon", href: "/favicon.svg", type: "image/svg+xml" },
  { rel: "icon", href: "/favicon.ico", sizes: "32x32" },
  { rel: "apple-touch-icon", href: "/apple-touch-icon.png" },
  { rel: "preload", href: "/fonts/fredoka.woff2", as: "font", type: "font/woff2", crossOrigin: "anonymous" },
];

export function Layout({ children }) {
  return (
    <html lang="en">
      <head>
        <meta charSet="utf-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover" />
        <meta name="theme-color" content="#1E64C8" />
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
