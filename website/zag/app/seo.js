// What a page tells search engines and link previews (owner, 2026-10-15: "Do those essentials"):
// its title and description, its one true address, and the card a shared link shows, with the
// social preview from brand/zag/ (copied in by scripts/post-build.mjs).
export const SITE = "https://zag-lang.org";
const IMAGE = { url: `${SITE}/social-preview.png`, width: "1280", height: "640", alt: "The Zag cloud logo on a blue sky: Very slow at the development, fast & safe at the final product." };

export function seo({ title, description, path }) {
  const url = SITE + path;
  return [
    { title },
    { name: "description", content: description },
    { tagName: "link", rel: "canonical", href: url },
    { property: "og:type", content: "website" },
    { property: "og:site_name", content: "Zag" },
    { property: "og:title", content: title },
    { property: "og:description", content: description },
    { property: "og:url", content: url },
    { property: "og:image", content: IMAGE.url },
    { property: "og:image:width", content: IMAGE.width },
    { property: "og:image:height", content: IMAGE.height },
    { property: "og:image:alt", content: IMAGE.alt },
    { name: "twitter:card", content: "summary_large_image" },
  ];
}
