# A member may have versions, as the own function may (design/collections.md, 2026-10-20/21): `versions round.near:`
# … `;versions round.near`, each version its parameters and its body, the call picked by how many arguments it has.
coll.fn round:
    versions round.near:
        [let $x: f64] -> f64:
            ret $x;
        ;version
        [let $x: f64, let $places: i32] -> f64:
            ret $x + to.f64($places);
        ;version
    ;versions round.near
;coll.fn round

MAIN:
    std::print(my.round.near(2.5), " ", my.round.near(2.5, 1), newln);
;MAIN
