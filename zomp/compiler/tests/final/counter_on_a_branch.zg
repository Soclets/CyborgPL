# A counter stepped on only some trips does not bound how many trips there
# are: for an odd N this loop goes round forever, and the sum overflows.
# The final build must not bound 'acc' by the counter.
extern.C fn exit ['code': int32];
extern.C fn getenv ['name': ptr[uint8]] -> ptr[uint8];
extern.C fn atoll ['s': ptr[uint8]] -> int64[min 0, max 1000];
fn read_n ['text': ptr[uint8]] -> int64[min 0, max 1000] or fail[missing, out_of_range] {
    if ptr.is_null['text'] { fail missing; }
    no_check let 'n' := atoll['text'] otherwise fail;
    return 'n';
}
fn run [] -> int32 or fail[missing, out_of_range] {
    let 'key' := "N";
    no_check let 'n' := read_n[getenv[lend 'key']] otherwise fail;
    let mut 'acc': int64 = 0;
    let mut 'i': int64 = 0;
    while 'i' at_most 'n' {
        if ('n' mod.floor 2) equal_to 0 { change 'i' = 'i' plus 1; }
        change 'acc' = 'acc' plus 1_000_000_000_000;
    }
    return to.int32['acc' mod.floor 256];
}
MAIN {
    let 'code' := run[] otherwise 2;
    no_check my.exit['code'];
}
