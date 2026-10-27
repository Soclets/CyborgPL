# A const in a body (owner, 2026-10-10: "I want it", then "`a`": any body, MAIN and functions): its
# value is known at compile time, nobody owns it (it is written bare, like a top-level const), it is
# never shadowed, and it is seen only in its block. `let immut` holds a value worked out while running.
const $base := 100;

fn tries [let $n: int.sign.32] -> int.sign.32:
    const $extra := 2;
    ret $n plus $extra;
end of fn tries

!MAIN:
    const $retries := 3;
    const $greeting: str.utf8 = "hi";
    const $limit := $base times $retries;
    std::print.shortest($greeting, " ", $retries, " ", $limit, " ", tries($retries), newln);
!end of MAIN
