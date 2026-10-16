# `*0.1` is exactly what is written, a deci (owner, 2026-10-02): deci64 under
# `:=`, the width of a typed place otherwise. 0.3 x 10 + 19.99 x 100 + 0.5 x 2
# = 3 + 1999 + 1 = 2003, 211 mod 256; the oracle agrees.
extern.C fn exit ['code': int32];
MAIN {
    let 'price' := exactly 19.99;
    let 'tenth' := exactly 0.1;
    let 'total' := 'tenth' plus exactly 0.2;
    let 'half': deci32 = exactly 0.5;
    let 'n': int32 = to.int32[('total' times 10) plus ('price' times 100) plus to.stored.deci64['half' times 2]] otherwise 0;
    no_check my.exit['n' mod.floor 256];
}
