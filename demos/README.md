# demos

Programs to build and look at, which no test runs (they open windows, wait for a person, or need a library this
machine may lack). Each is a kit; build one with `kayo build --final demos/<name> -o build/<name>`.

- `sdl_window`: a real window, SDL3 drawing "Hello World" until it is closed (owner, 2026-10-22). Needs SDL3
  (`brew install sdl3`).
- `hello_button`: a window saying Hello World, with a Quit button that lights up under the mouse and ends the program
  when clicked (owner, 2026-10-25). Needs SDL3.
- `mandelbrot`: the Mandelbrot set as a PPM image on stdout, 900 by 600 (owner, 2026-10-22: "Draw something using
  CyborgPL"). `build/mandelbrot > m.ppm`, then `sips -s format png m.ppm --out m.png`.
