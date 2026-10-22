# refuse: this `no_check` covers something that fails every time it runs
# A no_check the prover sees fail on every run is refused, with the value that breaks it (owner, 2026-10-22).
MAIN {
    let 'xs': std::list[i64] = [1, 2];
    no_check let 'v' := 'xs'[pos: 3];
    std::print.useful['v', newln];
}
