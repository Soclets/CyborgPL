# Another kit's `pub coll.handler`, used through its path, with arguments (owner, 2026-10-23): its arms read as if
# written at the call, each first declaring the parameters from the arguments.
procedure half_or [$n: integer.signed.32_bit, $d: integer.signed.32_bit] -> integer.signed.32_bit:
    let $h := halves::half($n) otherwise halves::half_failed.or_default(default: $d, label: "half_or");
    return $h;
end of procedure half_or

!MAIN:
    std::print.shortest(half_or(8, 0), " ", half_or(7, 100), " ", half_or(negative 4, 5 times 2), newline);
!end of MAIN
