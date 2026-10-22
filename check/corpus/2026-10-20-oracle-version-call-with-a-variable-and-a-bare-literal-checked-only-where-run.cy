# Finding 2026-10-20 (by hand, the limit design claude named with 7ab59e76): a version call with a
# variable beside a bare literal that two versions could take. mx['v', 5] with 'v' an i32: the 5
# fits both i32 and i64, so the call is refused (owner, 2026-10-20). In a branch never taken the
# oracle prints "before" and "after"; kayo refuses it when building ("a bare literal fits more than
# one; say which, `5: i64`"). The all-literal case is refused before running since 7ab59e76.
# Was an oracle limit, marked known (it picked a version for a call with a variable only when the call
# ran); fixed in oracle 22a24001: a variable typed by its declaration is picked before running.
# Moved to a `versions` block (owner, 2026-10-21; oracle 78a25324).
coll.fn mx {
    versions mx {
        ['a': i32, 'b': i32] -> i32 { return 'a' + 'b'; }
        ['a': i32, 'b': i64] -> i32 { return 'a' - to.i32['b']; }
    }
}
MAIN {
    let 'v': i32 = 4;
    let 'p': bool = to.i64[1] equal_to to.i64[2];
    std::print.useful["before", newln];
    if 'p' {
        std::print.useful[mx['v', 5], newln];
    }
    std::print.useful["after", newln];
}
