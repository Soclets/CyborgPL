# a function named `MAIN` is called by its path, never bare (owner, 2026-10-26)
fn MAIN [] -> i32 { return 1; }
MAIN {
    let $x := MAIN();
}
