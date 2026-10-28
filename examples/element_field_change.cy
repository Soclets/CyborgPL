# A field of a list's element changes in place, `change $bodies[pos: $i].vx -= …`: the record's field is
# checked as `change $b.vx to …` checks it, a `let` field refused.
rec Body:
    let name: str.utf8;
    var vx: f64;
;rec Body

MAIN:
    var $bodies = [Body[name = "sun", vx = 0.0], Body[name = "jupiter", vx = 1.5]];
    change $bodies[pos: 2].vx -= 0.25;
    change $bodies[pos: 1].vx to $bodies[pos: 2].vx x 2.0;
    for $b in lend $bodies:
        std::print($b.name, " ", $b.vx, newln);
    ;for
;MAIN
