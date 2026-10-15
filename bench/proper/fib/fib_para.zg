# In parallel: the top of the recursion as four `concur` tasks, each sending its part on a channel;
# below that, naive recursion as the program asks.
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
    if 'n' < 4 { return to.int32[fib['n'] mod.floor 256]; }
    # fib(n) = fib(n-2) + 2 fib(n-3) + fib(n-4): four tasks, each on its own channel
    let 'c1' := channel[int64];
    let 'c2' := channel[int64];
    let 'c3' := channel[int64];
    let 'c4' := channel[int64];
    concur with [lend 'c1', lend 'n'] { channel.send[lend 'c1', value fib['n' minus 2]]; }
    concur with [lend 'c2', lend 'n'] { channel.send[lend 'c2', value fib['n' minus 3]]; }
    concur with [lend 'c3', lend 'n'] { channel.send[lend 'c3', value fib['n' minus 3]]; }
    concur with [lend 'c4', lend 'n'] { channel.send[lend 'c4', value fib['n' minus 4]]; }
    let 'a' := channel.take[lend 'c1'] otherwise 0;
    let 'b' := channel.take[lend 'c2'] otherwise 0;
    let 'c' := channel.take[lend 'c3'] otherwise 0;
    let 'd' := channel.take[lend 'c4'] otherwise 0;
    # each part's low byte first: what comes off a channel could be any int64
    return to.int32[(('a' mod.floor 256) plus ('b' mod.floor 256) plus ('c' mod.floor 256) plus ('d' mod.floor 256)) mod.floor 256];
}

MAIN {
    let 'code' := run[] otherwise 2;
    no_check my.exit['code'];
}
