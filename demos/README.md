# demos

Programs to build and look at, which no test runs (they open windows, wait for a person, or need a library this
machine may lack). Each is a kit; build one with `kayo build --final demos/<name> -o build/<name>`.

- `sdl_window`: a real window, SDL3 drawing "Hello World" until it is closed (owner, 2026-10-22). Needs SDL3
  (`brew install sdl3`).
