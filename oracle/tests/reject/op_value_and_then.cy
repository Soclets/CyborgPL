# `op and.then` could never skip: the arguments are worked out first (owner, 2026-09-29)
MAIN {
    let 'g': fn[bool, bool] -> bool = op and.then;
}
