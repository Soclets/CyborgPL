# A window that says Hello World, with a button that quits the program (owner, 2026-10-25: "a GUI saying Hello World
# with a button that quits the program"). SDL3 draws it through CNC and C: the text with SDL's built-in debug font, the
# button as a filled rectangle that lights up under the mouse; a click on it, or closing the window, ends the program.
# Build and run it with `kayo build --final demos/hello_button -o build/hello_button && build/hello_button`.
%extern.C proc SDL_Init [let $flags: u32] -> bool;
%extern.C proc SDL_CreateWindow [let $title: ptr[u8], let $w: i32, let $h: i32, let $flags: u64] -> ptr.mut[u8];
%extern.C proc SDL_CreateRenderer [let $window: ptr.mut[u8], let $name: ptr[u8]] -> ptr.mut[u8];
%extern.C proc SDL_SetRenderScale [let $r: ptr.mut[u8], let $x: f32, let $y: f32] -> bool;
%extern.C proc SDL_SetRenderDrawColor [let $r: ptr.mut[u8], let $red: u8, let $green: u8, let $blue: u8, let $alpha: u8] -> bool;
%extern.C proc SDL_RenderClear [let $r: ptr.mut[u8]] -> bool;
%extern.C proc SDL_RenderFillRect [let $r: ptr.mut[u8], let $rect: ptr[f32]] -> bool;
%extern.C proc SDL_RenderRect [let $r: ptr.mut[u8], let $rect: ptr[f32]] -> bool;
%extern.C proc SDL_RenderDebugText [let $r: ptr.mut[u8], let $x: f32, let $y: f32, let $text: ptr[u8]] -> bool;
%extern.C proc SDL_RenderPresent [let $r: ptr.mut[u8]] -> bool;
%extern.C proc SDL_PollEvent [let $event: ptr.mut[u8]] -> bool;
%extern.C proc SDL_GetMouseState [let $x: ptr.mut[f32], let $y: ptr.mut[f32]] -> u32;
%extern.C proc SDL_Delay [let $ms: u32];
%extern.C proc SDL_DestroyRenderer [let $r: ptr.mut[u8]];
%extern.C proc SDL_DestroyWindow [let $window: ptr.mut[u8]];
%extern.C proc SDL_Quit [];

MAIN:
    let $title := %C.str.make("CyborgPL");
    let $hello := %C.str.make("Hello World");
    let $quit := %C.str.make("Quit");
    # SDL_Event is 128 bytes; its first 4 are the type: SDL_EVENT_QUIT is 0x100, SDL_EVENT_MOUSE_BUTTON_UP 0x402
    var $event: std::ls[u8] = std::ls.fill(value: 0, count: 128);
    # the mouse, in window points; the drawing is scaled by 2, so a point is half a drawn unit
    var $mx: std::ls[f32] = [0.0,];
    var $my: std::ls[f32] = [0.0,];
    # the button, an SDL_FRect: x, y, width, height, in drawn units (the window is 320 by 180 of them)
    let $button: std::ls[f32] := [120.0, 100.0, 80.0, 28.0];
    no_check:
        let $ok := SDL_Init(32);
        let $window := SDL_CreateWindow(lend $title, 640, 360, 0);
        let $r := SDL_CreateRenderer($window, ptr.null);
        let $scaled := SDL_SetRenderScale($r, 2.0: f32, 2.0: f32);
        var $running = true;
        while $running:
            let $buttons := SDL_GetMouseState(lend.mut $mx, lend.mut $my);
            let $x: f32 := $mx[pos: 1] div.near 2.0;
            let $y: f32 := $my[pos: 1] div.near 2.0;
            let $over := $x >= 120.0 and $x <= 200.0 and $y >= 100.0 and $y <= 128.0;
            while SDL_PollEvent(lend.mut $event):
                if $event[pos: 1] == 0 and $event[pos: 2] == 1:
                    change $running to false;
                ;if
                if $event[pos: 1] == 2 and $event[pos: 2] == 4 and $over:
                    change $running to false;
                ;if
            ;while
            let $c1 := SDL_SetRenderDrawColor($r, 20, 24, 40, 255);
            let $c2 := SDL_RenderClear($r);
            let $c3 := SDL_SetRenderDrawColor($r, 120, 220, 255, 255);
            let $d1 := SDL_RenderDebugText($r, 116.0: f32, 50.0: f32, lend $hello);
            if $over:
                let $c4 := SDL_SetRenderDrawColor($r, 200, 70, 70, 255);
            else:
                let $c5 := SDL_SetRenderDrawColor($r, 150, 50, 60, 255);
            ;if
            let $f := SDL_RenderFillRect($r, lend $button);
            let $c6 := SDL_SetRenderDrawColor($r, 240, 240, 245, 255);
            let $b := SDL_RenderRect($r, lend $button);
            let $d2 := SDL_RenderDebugText($r, 144.0: f32, 110.0: f32, lend $quit);
            let $p := SDL_RenderPresent($r);
            SDL_Delay(16);
        ;while
        SDL_DestroyRenderer($r);
        SDL_DestroyWindow($window);
        SDL_Quit();
    ;no_check
;MAIN
