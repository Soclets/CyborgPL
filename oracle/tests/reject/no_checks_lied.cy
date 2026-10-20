# NO_FUCKING_CHECKS vouches; the oracle reports a vouch that fails (2026-09-27)
MAIN {
    let 'z': i32 = 0;
    NO_FUCKING_CHECKS let 'q' := 1 div.floor 'z';
}
