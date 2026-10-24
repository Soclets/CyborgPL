# Bare groups between 2^63 and 2^64 beside a bitwise operator (check claude,
# seeds 25263 and 26090): worked exactly, then landing in the u64 they
# meet, as the oracle has it. 15803055715966303536 xor 5 and
# 10222921044520364505 or.bit 5, each mod 100: 41 + 9 = 50.
extern.C fn exit [$code: i32];
fn v [] -> u64 { return 5; }
MAIN {
    let $r: u64 = v() xor (15803055715966303544 minus 8);
    let $s: u64 = v() or.bit (3407640348173454835 times 3);
    no_check my.exit(to.i32(($r mod.floor 100) plus ($s mod.floor 100)));
}
