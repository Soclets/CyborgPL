# deci (owner, 2026-09-29): a literal with more digits than the width holds is an error, never rounded
MAIN {
    let 'q': d32 = 3.14159265;
}
