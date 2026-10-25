layout.C struct Pr { a: u32, b: u32 }
MAIN {
    let $prs: std::list[Pr] = [Pr[a = 1, b = 2],];
    let $p := $prs[pos: 1];
}
