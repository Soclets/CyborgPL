import { Links, Meta, Outlet, Scripts, ScrollRestoration } from "react-router";
import "./app.css";

export const links = () => [
  { rel: "icon", href: "/favicon.svg", type: "image/svg+xml" },
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

export default function App() {
  return <Outlet />;
}
