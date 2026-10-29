# A real window (owner, 2026-10-22: "show a real window"): SDL3 draws "Hello World" with its built-in debug font,
# until the window is closed. Through CNC and C; kayo builds it, linking SDL3 (`brew install sdl3`). The oracle
# cannot call real C libraries. In demos/, not examples/, since a test would open a window and wait: build and run
# it with `kayo build --final demos/sdl_window -o build/sdl_window && build/sdl_window`.
%extern.C proc SDL_Init [$flags: u32] -> bool;
%extern.C proc SDL_CreateWindow [$title: ptr[u8], $w: i32, $h: i32, $flags: u64] -> ptr.mut[u8];
%extern.C proc SDL_CreateRenderer [$window: ptr.mut[u8], $name: ptr[u8]] -> ptr.mut[u8];
%extern.C proc SDL_SetRenderScale [$r: ptr.mut[u8], $x: f32, $y: f32] -> bool;
%extern.C proc SDL_SetRenderDrawColor [$r: ptr.mut[u8], $red: u8, $green: u8, $blue: u8, $alpha: u8] -> bool;
%extern.C proc SDL_RenderClear [$r: ptr.mut[u8]] -> bool;
%extern.C proc SDL_RenderDebugText [$r: ptr.mut[u8], $x: f32, $y: f32, $text: ptr[u8]] -> bool;
%extern.C proc SDL_RenderPresent [$r: ptr.mut[u8]] -> bool;
%extern.C proc SDL_PollEvent [$event: ptr.mut[u8]] -> bool;
%extern.C proc SDL_Delay [$ms: u32];
%extern.C proc SDL_DestroyRenderer [$r: ptr.mut[u8]];
%extern.C proc SDL_DestroyWindow [$window: ptr.mut[u8]];
%extern.C proc SDL_Quit [];

MAIN:
    let $title := %C.str.make("CyborgPL");
    let $hello := %C.str.make("Hello World");
    let $from := %C.str.make("from CyborgPL, through SDL3");
    # SDL_Event is 128 bytes; its first 4 are the type, and SDL_EVENT_QUIT is 0x100
    var $event: std::ls[u8] = std::ls.fill(value: 0, count: 128);
    no_check:
        let $ok := SDL_Init(32);
        let $window := SDL_CreateWindow(lend $title, 640, 360, 0);
        let $r := SDL_CreateRenderer($window, ptr.null);
        let $scaled := SDL_SetRenderScale($r, 3.0: f32, 3.0: f32);
        var $running = true;
        while $running:
            while SDL_PollEvent(lend.mut $event):
                if $event[pos: 1] == 0 and $event[pos: 2] == 1:
                    change $running to false;
                ;if
            ;while
            let $c1 := SDL_SetRenderDrawColor($r, 20, 24, 40, 255);
            let $c2 := SDL_RenderClear($r);
            let $c3 := SDL_SetRenderDrawColor($r, 120, 220, 255, 255);
            let $d1 := SDL_RenderDebugText($r, 62.0: f32, 50.0: f32, lend $hello);
            let $c4 := SDL_SetRenderDrawColor($r, 200, 200, 210, 255);
            let $d2 := SDL_RenderDebugText($r, 4.0: f32, 70.0: f32, lend $from);
            let $p := SDL_RenderPresent($r);
            SDL_Delay(16);
        ;while
        SDL_DestroyRenderer($r);
        SDL_DestroyWindow($window);
        SDL_Quit();
    ;no_check
;MAIN
