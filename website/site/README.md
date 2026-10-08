# nerve-lang.com

The Nerve website: a mobile-first site that behaves like an app. Every screen
is pre-rendered to HTML at build time, then React takes over for app-style
navigation. Add it to a phone's home screen and it opens full-screen.

- **Stack** (owner, 2026-09-25): React + React Router 8 (pre-rendered, `ssr: false`)
  + Motion, hosted as static files on Cloudflare Workers (free, unlimited).
- **Content comes from the repo at build time**: every program in `examples/`
  with its real `.out`, highlighted by `../grammar/`; icons from `brand/`.
  Nothing is copied into this folder, so a rebuild picks up the latest.
- **Updates only when the owner asks.** Nothing deploys on push.
- **Home's comparison**: `app/lib/comparison.js` holds the dated copy, language
  versions and source links. Nerve has no version label; its design and current
  implementation are distinguished. Update the snapshot deliberately when asked.
- **Language artwork**: unaltered files and provenance are in
  `public/language-logos/`; the Nerve mark is copied from `brand/` at build time.
  `app/lib/language-artwork.js` supplies the marks and More's credits. Go uses the
  official Gopher mascot; the credits distinguish it from the Go wordmark.

```
cd website/site
npm install
npm run dev        # local, with hot reload
npm run build      # pre-render everything into build/client
npm run preview    # serve the build the way Cloudflare will (wrangler dev)
npm run deploy     # build, then publish to https://nerve-lang.com
```

`www.nerve-lang.com` → `nerve-lang.com` is a separate Worker in `../www-redirect/`.
