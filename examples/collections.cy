# Collections (owner, 2026-10-13): `coll.fn` writes a function and its dot narrowings in one
# block (the other kind, `coll.handler`, is in collection_handlers.cy). The own function carries
# the collection's name (no `fn`), a member with its own body is reached as `greet.loud`, and a preset,
# a member that only fills some of the own function's arguments, is one line: `greet.hi['who']` is
# `greet[lend 'who', "hi", 1]`. Private here, so no `pub` and no `bare.` mark; a `pub coll.fn`
# makes every member public under its one `bare.` mark.
coll.fn greet {
    greet [lent 'who': str.utf8, 'word': str.utf8, 'times': int32[min 1, max 5]] -> str.utf8 or fail[too_long] {
        if str.bytes[lend 'who'] > 20 { fail too_long; }
        let 'out' := str.empty;
        for 'i' from 1 to 'times' by 1 {
            change 'out' = 'out' concat 'word' concat ", ";
        }
        return 'out' concat 'who';
    }

    greet.hi ['word' = "hi", 'times' = 1];
    greet.cheer ['word' = "hooray", 'times' = 3];
    greet.loud [lent 'who': str.utf8] -> str.utf8 { return std::str.upper[lend 'who']; }
}

# A template (owner, 2026-10-13) makes members whose names follow a pattern. It loops with the usual
# `for` forms over values known before building, its first line `named` works out each member's name
# like text, and `kayo generate` (here `oracle --generate`) writes what it makes into this file,
# under the `#generated_N_lines_below` marker. The build only reads that part, and refuses it when it
# no longer matches the template; it is never edited by hand. Members that only fix an argument are
# presets, as `scale.times_2` here (Good practice 14): a template is for what a preset cannot make,
# such as a member whose signature changes with the value, as `zero`'s below.
const 'big' := 100;

coll.fn scale {
    scale ['x': int32, 'by': int32] -> int32 { return 'x' x 'by'; }
    scale.times_2 ['by' = 2];
    scale.times_10 ['by' = 10];
    scale.times_100 ['by' = 'big'];
}

# A template over types: each member returns its own type, which no preset can do. A type's name is
# text in `named`.
coll.fn zero {
    for 'T' in [int16, int64] {
        named "of_" concat 'T';
        [] -> 'T' { return 0; }
    }
    #generated_2_lines_below
    zero.of_int16 [] -> int16 { return 0; }
    zero.of_int64 [] -> int64 { return 0; }
}

# `named` may call a function, run while generating, when a name needs logic (owner, 2026-10-13:
# `named size_name['n'];`). A type is its name's text there.
fn width_name ['t': str.utf8] -> str.utf8 {
    if 't' equal_to "int16" { return "small"; }
    return "big";
}

coll.fn twice {
    for 'T' in [int16, int64] {
        named width_name['T'];
        ['x': 'T'] -> 'T' { return 'x' x 2; }
    }
    #generated_2_lines_below
    twice.small ['x': int16] -> int16 { return 'x' x 2; }
    twice.big ['x': int64] -> int64 { return 'x' x 2; }
}

# A member whose body is written elsewhere (owner, 2026-10-19): `area.square = square_area;` is that function under
# the member's name, its path read as any path from here (`shapes.square.area` in a kit's root file).
fn square_area ['s': int32] -> int32 { return 's' x 's'; }

coll.fn area {
    area ['w': int32, 'h': int32] -> int32 { return 'w' x 'h'; }
    area.square = square_area;
}

MAIN {
    let 'name' := "Ann";
    let 'a' := greet[lend 'name', "hello", 2] otherwise "?";
    let 'b' := greet.hi[lend 'name'] otherwise "?";
    let 'c' := greet.cheer[lend 'name'] otherwise "?";
    std::print.useful['a', newln, 'b', newln, 'c', newln, greet.loud[lend 'name'], newln];
    let 'long' := "a name much longer than twenty bytes";
    let 'd' := greet.hi[lend 'long'] otherwise "too long";
    std::print.useful['d', newln];
    std::print.useful[scale[7, 3], " ", scale.times_2[7], " ", scale.times_10[7], " ", scale.times_100[7], " ", zero.of_int64[], " ", twice.small[300], " ", twice.big[5_000_000_000], newln];
    std::print.useful[area[3, 4], " ", area.square[5], newln];
}
