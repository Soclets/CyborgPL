# refuse: the value can be outside what a `u64` holds
# An exact answer beside `xor` lands in the operator's width first: -8 is
# not a u64, and was taken as its bit pattern (found with check claude's
# seeds 25263 and 26090).
%extern.C fn exit [$code: i32];
fn v [] -> u64 { return 5; }
fn w [] -> u64 { return 5; }
MAIN {
    let $r: u64 = v() xor (w() minus 13);
    no_check my.exit(to.i32($r mod.floor 100));
}
