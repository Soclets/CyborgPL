# env CHK_A 0
# Finding 2026-09-23 (generator seed 87 with inputs, shrunk): Precog loses a
# known value through a bitwise operator (5 and.bit 0 is 0), so this `check`
# is refused although 'n' x 0 is always 0.
extern.C fn getenv ['name': ptr[u8]] -> ptr[u8];
extern.C fn strlen ['s': ptr[u8]] -> u64;
MAIN {
    let 'k' := "CHK_A";
    no_check let 'n': u64 = strlen[getenv[lend 'k']];
    let 'z': u8 = 5 and.bit 0;
    let 'r': u64 = 0;
    check { change 'r' = 'n' times 'z'; }
}
