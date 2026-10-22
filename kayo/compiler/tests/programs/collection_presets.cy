# Collections (owner, 2026-10-13): a preset fills some of the main
# function's arguments, passes the rest as the main function takes them
# (`lend` for the lent list) and fails as it fails; a member may point at a
# function written elsewhere (owner, 2026-10-19), in a collection with its
# own function and in one without. 73 + 12 + 100 (the empty list's
# failure) + 7 is 192, + 9 (`mix.square[3]`) + 4 (`only.twice[2]`) is 205.
extern.C fn exit ['code': i32];

fn square ['x': i32] -> i32 { return 'x' times 'x'; }
fn double ['x': i32] -> i32 { return 'x' times 2; }

coll.fn mix {
    mix [lent 'xs': std::list[i32], 'a': i32, 'b': i32] -> i32 or fail[empty] {
        if std::list.length[lend 'xs'] equal_to 0 { fail empty; }
        return 'a' times 10 plus 'b';
    }
    mix.seven ['a' = 7];
    mix.both ['a' = 1, 'b' = 2];
    mix.square = square;
    for 'n' [from: 1, to: 2, by: 1] {
        named "plus_" concat to.stored.str.utf8['n'];
        ['x': i32] -> i32 { return 'x' plus 'n'; }
    }
    #generated_2_lines_below
    mix.plus_1 ['x': i32] -> i32 { return 'x' plus 1; }
    mix.plus_2 ['x': i32] -> i32 { return 'x' plus 2; }
}

coll.fn only {
    only.twice = double;
}

MAIN {
    let 'xs': std::list[i32] = [1, 2];
    let 'none': std::list[i32] = std::list.empty;
    let 'a' := mix.seven[lend 'xs', 3] otherwise 0;
    let 'b' := mix.both[lend 'xs'] otherwise 0;
    let 'c' := mix.both[lend 'none'] otherwise 100;
    let 't' := 'a' plus 'b' plus 'c' plus mix.plus_2[5] plus mix.square[3] plus only.twice[2];
    no_check my.exit[to.i32['t']];
}
