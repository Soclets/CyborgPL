# A copy of examples/kits/areas under the prefix `shapes`, as Synthia makes one for
# `kit.areas2.prefix = shapes` (owner, 2026-09-30); `my.` here is this kit's root.


pub const $pi: float.bin.64 = 3.14159;

pub rec Circle:
    radius: float.bin.64;
end of rec Circle

pub fn circle [lent $c: Circle] -> float.bin.64:
    ret $c.radius times $c.radius times my.$pi;
end of fn circle

pub.kit fn internal [] -> int.sign.32:
    ret 2;
end of fn internal

fn hidden [] -> int.sign.32:
    ret 1;
end of fn hidden

pub module shapes:
    pub fn square [$s: int.sign.32] -> int.sign.32:
        ret $s times $s;
    end of fn square
end of module shapes

# A kit's own word operator with a narrowed form (owner, 2026-10-02/03): users write it bare or
# with this kit's path, `areas::mean.low`.
# its levels apply wherever `mean` is used, in other kits too (owner, 2026-10-03)
precedence.low_first:
    level 1 [ plus ] left_first
    level 2 [ times, mean, mean.low ] left_first
end of precedence.low_first

pub op mean [$a: float.bin.64, $b: float.bin.64] -> float.bin.64:
    ret ($a plus $b) div.near 2.0;
end of op mean
pub op mean.low [$a: float.bin.64, $b: float.bin.64] -> float.bin.64:
    ret ($a plus $b) div.floor 2.0;
end of op mean.low
