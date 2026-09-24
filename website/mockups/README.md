# Website mockups — design experiments

These are **design explorations, not the website.** Nothing here is final: the
stack, the layout and the palette are all still being tried out (owner,
2026-09-24). Expect any of it to be thrown away.

## What's here

- `gate/index.html` — the first page: a CLI prompt. Type `proceed` and press
  Enter to go on. Wrong input answers in CyborgPL's own diagnostic shape.
- `gate/construction.html` — the "Under construction" page it leads to. Two
  striped bands slide in opposite directions; **Enter anyway** opens the app.
- `app/` — the site as a desktop app: one window on a desktop, with a sidebar,
  a Welcome page and the real `examples/` with their real output. The window
  can be moved, snapped to either half of the screen, zoomed, resized,
  minimized to the dock, closed and reopened, and put in full screen. React +
  Vite.

All three use Solarized Dark. Code is highlighted with the grammar in
`../grammar/`.

## Running them

```
cd website/mockups/app
npm install
npm run build          # renders examples/ into src/data.json, then builds app/dist/
cd ..
python3 -m http.server 8765
```

Then open `http://localhost:8765/gate/index.html`. **Enter anyway** expects the
app to have been built into `app/dist/` first.
