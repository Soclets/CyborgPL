# Another kit's `pub coll.handler`, used through its path, with arguments (owner, 2026-10-23): its arms read as if
# written at the call, each first declaring the parameters from the arguments.
proc half_or [$n: int.sign.32, $d: int.sign.32] -> int.sign.32:
    let $h := halves::half($n) otherwise halves::half_failed.or_default(default: $d, label: "half_or");
    ret $h;
end of proc half_or

!MAIN:
    std::print.shortest(half_or(8, 0), " ", half_or(7, 100), " ", half_or(neg 4, 5 times 2), newln);
!end of MAIN
