# A copy of examples/kits/areas under the prefix `shapes`, as Synthia makes one for
# `kit.areas2.prefix = shapes` (owner, 2026-09-30); `my.` here is this kit's root.


pub const $pi: f64 := 3.14159;

pub rec Circle:
    var radius: f64;
;rec Circle

pub fn circle [lent $c: Circle] -> f64:
    ret $c.radius x $c.radius x my.$pi;
;fn circle

pub.kit fn internal [] -> i32:
    ret 2;
;fn internal

fn hidden [] -> i32:
    ret 1;
;fn hidden

pub module shapes:
    pub fn square [$s: i32] -> i32:
        ret $s x $s;
    ;fn square
;module shapes

# A kit's own word operator with a narrowed form (owner, 2026-10-02/03): users write it bare or
# with this kit's path, `areas::mean.low`.
# its levels apply wherever `mean` is used, in other kits too (owner, 2026-10-03)
precedence.low_first:
    level 1 [+] left_first;
    level 2 [x, mean, mean.low] left_first;
;precedence.low_first

pub op mean [lent $a: f64, lent $b: f64] -> f64:
    ret ($a + $b) div.near 2.0;
;op mean
pub op mean.low [lent $a: f64, lent $b: f64] -> f64:
    ret ($a + $b) div.floor 2.0;
;op mean.low
