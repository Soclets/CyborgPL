# Finding 2026-10-20 (by hand, the limit design claude named with 7ab59e76): a version call with a
# variable beside a bare literal that two versions could take. mx['v', 5] with 'v' an int32: the 5
# fits both int32 and int64, so the call is refused (owner, 2026-10-20). In a branch never taken the
# oracle prints "before" and "after"; kayo refuses it when building ("a bare literal fits more than
# one; say which, `5: int64`"). The all-literal case is refused before running since 7ab59e76.
coll.fn mx {
    mx ['a': int32, 'b': int32] -> int32 { return 'a' + 'b'; }
    mx ['a': int32, 'b': int64] -> int32 { return 'a' - to.int32['b']; }
}
MAIN {
    let 'v': int32 = 4;
    let 'p': bool = to.int64[1] equal_to to.int64[2];
    std::print.useful["before", newln];
    if 'p' {
        std::print.useful[mx['v', 5], newln];
    }
    std::print.useful["after", newln];
}
