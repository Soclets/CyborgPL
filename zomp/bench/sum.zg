# The sum of i x i mod 7 for i = 1 to N, kept below 1_000_000_007; the exit
# code is the sum's low byte. N comes from the environment and must be 0 to
# 1_000_000_000: `atoll`'s answer is checked at the door, and anything else
# ends the program with exit code 2, as does no N at all.
extern.C fn exit ['code': int32];
extern.C fn getenv ['name': ptr[uint8]] -> ptr[uint8];
extern.C fn atoll ['s': ptr[uint8]] -> int64[min 0, max 1_000_000_000];

# N, from the text in the environment: missing when there is none, and out
# of range when it is not a number this program takes.
fn read_n ['text': ptr[uint8]] -> int64[min 0, max 1_000_000_000] or fail[missing, out_of_range] {
    if ptr.is_null['text'] { fail missing; }
    no_check let 'n' := atoll['text'] otherwise fail;
    return 'n';
}

fn run [] -> int32 or fail[missing, out_of_range] {
    let 'key' := "N";
    no_check let 'n' := read_n[getenv[lend 'key']] otherwise fail;
    let mut 'acc': int64 = 0;
    let mut 'i': int64 = 1;
    while 'i' at_most 'n' {
        change 'acc' = ('acc' plus (('i' times 'i') mod.floor 7)) mod.floor 1_000_000_007;
        change 'i' = 'i' plus 1;
    }
    return to.int32['acc' mod.floor 256];
}

MAIN {
    let 'code' := run[] otherwise 2;
    no_check my.exit['code'];
}
