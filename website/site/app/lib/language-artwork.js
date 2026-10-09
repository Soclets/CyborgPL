import { REPO_URL } from "./content.js";

export const LANGUAGE_ARTWORK = {
  nerve: {
    asset: "/language-logos/nerve.svg", title: "Nerve",
    credit: "The Nerve icon: a cell body with nerves bursting from it, on its black tile. Copyright 2026 Tankun Sriket. Nerve and its logo are project trademarks.",
    links: [
      { label: "Project artwork", href: `${REPO_URL}/blob/main/brand/nerve/icon.svg` },
      { label: "Usage terms", href: "https://nerve-lang.com/licence" },
    ],
  },
  rust: {
    asset: "/language-logos/rust.svg", title: "Rust",
    credit: "Rust logo, owned by the Rust Foundation. CC BY 4.0. Artwork unmodified.",
    links: [
      { label: "Official artwork", href: "https://github.com/rust-lang/rust-artwork/tree/main/logo" },
      { label: "CC BY 4.0", href: "https://creativecommons.org/licenses/by/4.0/" },
      { label: "Trademark terms", href: "https://rustfoundation.org/policy/rust-trademark-policy/" },
    ],
  },
  go: {
    asset: "/language-logos/go.svg", title: "Go — Gopher mascot",
    credit: "Go Gopher by Renee French. CC BY 4.0. Artwork unmodified.",
    links: [
      { label: "Creator & artwork", href: "https://go.dev/blog/gopher" },
      { label: "CC BY 4.0", href: "https://creativecommons.org/licenses/by/4.0/" },
      { label: "Brand terms", href: "https://go.dev/brand" },
    ],
  },
  python: {
    asset: "/language-logos/python.svg", title: "Python", trademark: true,
    credit: "Python logo by Tim Parkin, owned by the Python Software Foundation. Python and its logo are PSF trademarks. Artwork unmodified.",
    links: [
      { label: "Official artwork", href: "https://www.python.org/community/logos/" },
      { label: "Designer credit", href: "https://www.python.org/psf/trademarks-faq/" },
      { label: "Usage terms", href: "https://www.python.org/psf/trademarks/" },
    ],
  },
};
