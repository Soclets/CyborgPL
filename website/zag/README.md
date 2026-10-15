# The Zag website

For zag-lang.org (owner, 2026-10-15: "Let's make the new website"). A fresh design: a moving blue sky with white clouds. The build is like nerve-lang.com's: React Router, every page built to HTML ahead of time, served as static files by Cloudflare Workers.

```
npm install
npm run build
npx wrangler dev        # preview at http://localhost:8787
```

Not deployed yet. It goes up, with the zag-lang.org route added to `wrangler.jsonc`, only when the owner says so.

- `public/logo.svg` and `public/favicon.svg` are copies of `brand/zag/logo.svg` and `brand/zag/logo-square.svg`.
- `public/fonts/fredoka.woff2` is Fredoka (SIL Open Font License 1.1, `fredoka-OFL.txt` beside it), cut down to Latin.
