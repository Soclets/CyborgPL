# A collection says its function's labels and defaults (owner, 2026-10-21): `labels ['text' as value]` gives a
# parameter its label, written `value:` in the call; `defaults ['times' = 1]` what a left-out argument is. A `pub`
# function says whether its labels keep their order, `labels.in_order` or `labels.any_order`; a private one's plain
# `labels` keeps it. The arguments without a label come first, in place. Versions go in a `versions` block.
coll.fn say {
    labels ['text' as value, 'times' as count];
    defaults ['times' = 1];
    say ['n': i32, 'text': str.utf8, 'times': i32] -> str.utf8 {
        return to.stored.str.utf8['n'] concat ": " concat 'text' concat " x" concat to.stored.str.utf8['times'];
    }
}

pub coll.fn box {
    labels.any_order ['w' as width, 'h' as height, 'b' as border];
    defaults ['b' = 0];
    box ['w': u32, 'h': u32, 'b': u32] -> u32 { return ('w' + 'b' + 'b') x ('h' + 'b' + 'b'); }
}

coll.fn area {
    versions area {
        ['side': i64] -> i64 { return 'side' x 'side'; }
        ['w': i64, 'h': i64] -> i64 { return 'w' x 'h'; }
    }
}

MAIN {
    std::print.useful[say[2, value: "hi"], newln];
    std::print.useful[say[3, value: "yo", count: 4], newln];
    std::print.useful[box[height: 4, width: 10], " ", box[border: 1, width: 10, height: 4], newln];
    std::print.useful[area[3], " ", area[3, 4], newln];
}
