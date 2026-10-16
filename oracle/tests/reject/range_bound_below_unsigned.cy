# an unsigned type's range cannot start below zero (owner, 2026-09-23)
MAIN {
    let 'v': uint8[min -1, max 5] = 5;
}
