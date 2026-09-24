# cyborgpl.dev placeholder

What's live at https://cyborgpl.dev until the real site exists: one line on a
white page (owner, 2026-09-24).

- `public/` — the page. `wrangler.jsonc` — the `cyborgpl-website` Worker
  (static files only; serves `cyborgpl.dev`).
- `www-redirect/` — the `cyborgpl-www-redirect` Worker: sends
  `www.cyborgpl.dev` to `cyborgpl.dev`, keeping the path.

Deploy (only when the owner asks — the site is not updated on every push):

```
cd website/placeholder && npx wrangler deploy
cd www-redirect && npx wrangler deploy
```
