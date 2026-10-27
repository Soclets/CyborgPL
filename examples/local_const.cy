# A const in a body (owner, 2026-10-10: "I want it", then "`a`": any body, MAIN and functions): its
# value is known at compile time, nobody owns it (it is written bare, like a top-level const), it is
# never shadowed, and it is seen only in its block. `let immut` holds a value worked out while running.
constant $base := 100;

function tries [$n: integer.signed.32_bit] -> integer.signed.32_bit:
    constant $extra := 2;
    return $n plus $extra;
end of function tries

!MAIN:
    constant $retries := 3;
    constant $greeting: string.utf8 = "hi";
    constant $limit := $base times $retries;
    std::print.shortest($greeting, " ", $retries, " ", $limit, " ", tries($retries), newline);
!end of MAIN
