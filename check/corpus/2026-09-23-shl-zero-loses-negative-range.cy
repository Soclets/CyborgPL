# env CHK_A 5
# Finding 2026-09-23 (generator seed 58, shrunk structurally then by hand):
# Precog loses the interval of a negative value through `shl 0`, so a check
# that 1 + 'v' fits i16 is refused; without the shl it is proven.
extern.C fn getenv ['name': ptr[u8]] -> ptr[u8];
extern.C fn strlen ['s': ptr[u8]] -> u64;
MAIN {
    let 'k' := "CHK_A";
    no_check let 'n': u64 = strlen(getenv(lend 'k'));
    let 'v': i16[max: -91] = (to.i16('n' mod.floor 32768) mod.floor 32677) minus 32767;
    check let 'a': i16 = 1 plus ('v' shl 0);
}
