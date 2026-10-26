# Finding 2026-10-25 (check metal, the struct cases: 16 of 300 seeds): `print.useful` of an f32 is the shortest
# decimal that reads back as that f32 (owner, 2026-09-14). Printed straight from a struct's field, the oracle
# gives the shortest that reads back as an f64: -3080.765625 where the f32 itself prints -3080.7656, for a
# `layout.C` struct and a plain one alike; copied into a typed f32 first, it prints right. kayo prints
# -3080.7656 each time.
%layout.C struct Pr { a: f32, b: f32 }
struct Q { a: f32 }
MAIN {
    let $v: f32 = -3080.765625;
    let $p := Pr[a = $v, b = 1.5];
    let $q := Q[a = $v];
    std::print.useful($v, " ", $p.a, " ", $q.a, newln);
    let $w: f32 = $p.a;
    std::print.useful($w, newln);
}
