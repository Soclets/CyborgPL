# A copy of examples/kits/areas under the prefix `shapes`, as Synthia makes one for
# `kit.areas2.prefix = shapes` (owner, 2026-09-30); `my.` here is this kit's root.

bare.kit_prefix.never [fn, struct];
bare.kit_prefix.optional [op];

pub const 'pi': f64 = 3.14159;

pub struct Circle { radius: f64 }

pub fn circle [lent 'c': Circle] -> f64 {
    return 'c'.radius x 'c'.radius x my.'pi';
}

pub.kit fn internal [] -> i32 { return 2; }

fn hidden [] -> i32 { return 1; }

pub module shapes {
    pub fn square ['s': i32] -> i32 { return 's' x 's'; }
}

# A kit's own word operator with a narrowed form (owner, 2026-10-02/03): users write it bare or
# with this kit's path, `areas::mean.low`.
# its levels apply wherever `mean` is used, in other kits too (owner, 2026-10-03)
precedence.low_first {
    level 1 { + } left_first
    level 2 { times, mean, mean.low } left_first
}

pub op mean ['a': f64, 'b': f64] -> f64 { return ('a' + 'b') div.near 2.0; }
pub op mean.low ['a': f64, 'b': f64] -> f64 { return ('a' + 'b') div.floor 2.0; }
