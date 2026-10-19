# The same program with `no_cheat` in front of the work (owner, 2026-10-18/19): every call and loop under
# it really runs, so kayo may not fold it, drop it, compute look-alike calls once, or use Precog's tables.
# For the run-time bench's like-for-like rows (owner: "run a fresh one, with cheat and no_cheat").
# A list of N numbers (i mod 1000 for i = 1 to N), then summed ten times;
# the exit code is the sum's low byte. N comes from the environment and must
# be 0 to 100_000_000; anything else ends the program with exit code 2.
extern.C fn exit ['code': int32];
extern.C fn getenv ['name': ptr[uint8]] -> ptr[uint8];
extern.C fn atoll ['s': ptr[uint8]] -> int64[min 0, max 100_000_000];

# N, from the text in the environment: missing when there is none, and out
# of range when it is not a number this program takes.
fn read_n ['text': ptr[uint8]] -> int64[min 0, max 100_000_000] or fail[missing, out_of_range] {
    if ptr.is_null['text'] { fail missing; }
    no_check let 'n' := atoll['text'] otherwise fail;
    return 'n';
}

fn run [] -> int32 or fail[missing, out_of_range] {
    let 'key' := "N";
    no_check let 'n' := read_n[getenv[lend 'key']] otherwise fail;
    let mut 'xs': list[int64] = [];
    for 'i': int64 from 1 to 'n' by 1 {
        list.append[lend.mut 'xs', value 'i' mod.floor 1000];
    }
    let mut 'acc': int64 = 0;
    for 'r': int64 from 1 to 10 by 1 {
        for 'x' in lend 'xs' {
            change 'acc' = 'acc' plus 'x';
        }
    }
    return to.int32['acc' mod.floor 256];
}

MAIN {
    no_cheat let 'code' := run[] otherwise 2;
    no_check my.exit['code'];
}
