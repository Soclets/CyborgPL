import { ICON } from "./icon-hash.js";
// What search engines and link previews read: a description, the one true address of the
// page, and the brand avatar as the preview image. Every page's HTML also carries its full
// content — computers only hide it (root.jsx), so crawlers and people get the same page.
export const SITE = "https://nerve-lang.com";

export function seo({ title, description, path }) {
  const url = SITE + (path === "/" ? "/" : path);
  return [
    { title },
    { name: "description", content: description },
    { tagName: "link", rel: "canonical", href: url },
    { property: "og:type", content: "website" },
    { property: "og:site_name", content: "Nerve" },
    { property: "og:title", content: title },
    { property: "og:description", content: description },
    { property: "og:url", content: url },
    { property: "og:image", content: `${SITE}/icons/app-${ICON}-512.png` },
    { name: "twitter:card", content: "summary" },
  ];
}
