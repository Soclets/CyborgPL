# One name, several versions, picked by how many arguments and their types (owner, 2026-10-20),
# written together in a `coll.fn` (owner: b), in a `versions` block that names them once (owner, 2026-10-21).
coll.fn describe {
    versions describe {
        ['n': i64] -> str.utf8 { return "a whole number"; }
        ['x': f64] -> str.utf8 { return "a binary float"; }
        [lent 's': str.utf8] -> str.utf8 { return "a text"; }
    }
}

coll.fn width {
    versions width {
        ['n': i16] -> str.utf8 { return "16 bits"; }
        ['n': i64] -> str.utf8 { return "64 bits"; }
        ['x': f32] -> str.utf8 { return "a f32"; }
        ['x': f64] -> str.utf8 { return "a f64"; }
    }
}

coll.fn which {
    versions which {
        ['x': f32] -> str.utf8 { return "f32"; }
        ['x': f64] -> str.utf8 { return "f64"; }
    }
}

coll.fn area {
    versions area.rect {
        ['side': i64] -> i64 { return 'side' x 'side'; }
        ['w': i64, 'h': i64] -> i64 { return 'w' x 'h'; }
    }
}

MAIN {
    let 'n': i64 = 7;
    let 'x': f64 = 2.5;
    let 'name' := "Ann";
    std::print.useful[describe['n'], newln];
    std::print.useful[describe['x'], newln];
    std::print.useful[describe[lend 'name'], newln];
    std::print.useful[area.rect[3], " ", area.rect[3, 4], newln];
    let 'small': i16 = 5;
    let 'half': f32 = 0.5;
    let 'y': f64 = 2.5;
    std::print.useful[which[0.0 - 'y'], " ", which[-'half'], " ", which[('half' x 2.0)], newln];
    std::print.useful[width['small'], ", ", width['n'], ", ", width['half'], ", ", width['x'], ", ", width[2.5: f32], newln];
}
