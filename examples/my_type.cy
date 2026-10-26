# A module names the root's types through `my.` (owner, 2026-10-06: `my.` is the root), as std's `str.cy` names
# `my.casing`: in a parameter, a return type and a value built.
enum Color { red, blue }
struct P { x: i32 }

module paint {
    pub fn name [$c: my.Color] -> str.utf8 {
        match $c {
            red { return "red"; }
            blue { return "blue"; }
        }
    }
    pub fn twice [$p: my.P] -> my.P { return my.P[x = $p.x x 2]; }
}

MAIN {
    let $q := paint.twice(P[x = 4]);
    std::print.useful(paint.name(Color.blue), " ", $q.x, newln);
}
