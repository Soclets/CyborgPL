# a C struct's ranged field is proven where Cyborg fills it in (2026-09-27)
%layout.C struct Rect { w: i32[min: 0, max: 4096], h: i32[min: 0, max: 4096] }
MAIN {
    let $r := Rect[w = 5000, h = 1];
}
