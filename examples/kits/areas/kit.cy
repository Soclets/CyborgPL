# A small kit used by examples/uses_a_kit (a dependency: it has no MAIN). Only `pub` items reach
# another kit; `pub.kit` and private ones stay home. `my.` here is this kit's root.

bare.never [fn, struct];
bare.optional [op];

pub const 'pi': bin64 = 3.14159;

pub struct Circle { radius: bin64 }

pub fn circle [lent 'c': Circle] -> bin64 {
    return 'c'.radius x 'c'.radius x my.'pi';
}

pub.kit fn internal [] -> int32 { return 2; }

fn hidden [] -> int32 { return 1; }

pub module shapes {
    pub fn square ['s': int32] -> int32 { return 's' x 's'; }
}

# A kit's own word operator with a narrowed form (owner, 2026-10-02/03): users write it bare or
# with this kit's path, `areas::mean.low`.
# its levels apply wherever `mean` is used, in other kits too (owner, 2026-10-03)
precedence.low_first {
    level 1 { + } left_first
    level 2 { times, mean, mean.low } left_first
}

pub op mean ['a': bin64, 'b': bin64] -> bin64 { return ('a' + 'b') div.near 2.0; }
pub op mean.low ['a': bin64, 'b': bin64] -> bin64 { return ('a' + 'b') div.floor 2.0; }
