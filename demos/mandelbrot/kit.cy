# The Mandelbrot set (owner, 2026-10-22: "Draw something using CyborgPL"): each pixel's colour from how fast its
# point escapes, printed as a PPM image. Build, then `build/mandelbrot > m.ppm` and open or convert it
# (`sips -s format png m.ppm --out m.png`). 900 by 600, about 1.2 s.
MAIN {
    let $w: i64 = 900;
    let $h: i64 = 600;
    let $most: i64 = 120;
    std::print.useful("P3", newln, $w, " ", $h, newln, 255, newln);
    for $py: i64 [from: 0, to: $h - 1, by: 1] {
        for $px: i64 [from: 0, to: $w - 1, by: 1] {
            no_check {
                let $cx: f64 = -2.4 + 3.6 x (to.f64($px) div.near 900.0);
                let $cy: f64 = -1.2 + 2.4 x (to.f64($py) div.near 600.0);
                let mut $x: f64 = 0.0;
                let mut $y: f64 = 0.0;
                let mut $i: i64 = 0;
                while $i < $most and.then ($x x $x + $y x $y) < 4.0 {
                    let $nx: f64 = $x x $x - $y x $y + $cx;
                    change $y = 2.0 x $x x $y + $cy;
                    change $x = $nx;
                    change $i = $i + 1;
                }
                if $i equal_to $most {
                    std::print.useful("0 0 0", newln);
                } else {
                    # the classic smooth palette: t from 0 to 1, deep blue far out, gold and white near the edge
                    let $t: f64 = to.f64($i) div.near to.f64($most);
                    let $u: f64 = 1.0 - $t;
                    let $r: i64 = to.i64(std::round.floor(9.0 x $u x $t x $t x $t x 255.0)) otherwise 0;
                    let $g: i64 = to.i64(std::round.floor(15.0 x $u x $u x $t x $t x 255.0)) otherwise 0;
                    let $b: i64 = to.i64(std::round.floor(8.5 x $u x $u x $u x $t x 255.0)) otherwise 0;
                    std::print.useful($r, " ", $g, " ", $b, newln);
                }
            }
        }
    }
}
