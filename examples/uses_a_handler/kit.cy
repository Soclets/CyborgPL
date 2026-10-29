# Another kit's `pub coll.handler`, used through its path, with arguments (owner, 2026-10-23): its arms read as if
# written at the call, each first declaring the parameters from the arguments.
proc half_or [let $n: i32, let $d: i32] -> i32:
    let $h := halves::half($n) ?? halves::half_failed.*or_default(default: $d, label: "half_or");
    ret $h;
;proc half_or

MAIN:
    std::print(half_or(8, 0), " ", half_or(7, 100), " ", half_or(-4, 5 x 2), newln);
;MAIN
