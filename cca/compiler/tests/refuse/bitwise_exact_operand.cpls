# refuse: the value can be outside what a `uint64` holds
# An exact answer beside `xor` lands in the operator's width first: -8 is
# not a uint64, and was taken as its bit pattern (found with check claude's
# seeds 25263 and 26090).
extern.C fn exit ['code': int32];
fn v [] -> uint64 { return 5; }
fn w [] -> uint64 { return 5; }
MAIN {
    let 'r': uint64 = v[] xor (w[] minus 13);
    no_check my.exit[to.int32['r' mod.floor 100]];
}
