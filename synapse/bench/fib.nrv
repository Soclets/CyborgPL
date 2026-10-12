# The Nth Fibonacci number, recursively; the exit code is its low byte. N
# comes from the environment and must be 0 to 92 — fib(93) does not fit an
# int64 — and anything else ends the program with exit code 2, as does no N at all.
extern.C fn exit ['code': int32];
extern.C fn getenv ['name': ptr[uint8]] -> ptr[uint8];
extern.C fn atoll ['s': ptr[uint8]] -> int64[min 0, max 92];

# N, from the text in the environment: missing when there is none, and out
# of range when it is not a number this program takes.
fn read_n ['text': ptr[uint8]] -> int64[min 0, max 92] or fail[missing, out_of_range] {
    if ptr.is_null['text'] { fail missing; }
    no_check let 'n' := atoll['text'] otherwise fail;
    return 'n';
}

fn fib ['n': int64[min 0, max 92]] -> int64 {
    if 'n' < 2 { return 'n'; }
    return fib['n' minus 1] plus fib['n' minus 2];
}

fn run [] -> int32 or fail[missing, out_of_range] {
    let 'key' := "N";
    no_check let 'n' := read_n[getenv[lend 'key']] otherwise fail;
    return to.int32[fib['n'] mod.floor 256];
}

MAIN {
    let 'code' := run[] otherwise 2;
    no_check my.exit['code'];
}
