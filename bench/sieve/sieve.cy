# The primes up to N, by the sieve of Eratosthenes; the exit code is their
# count's low byte. N comes from the environment and must be 0 to
# 200_000_000; anything else ends the program with exit code 2, as does no N at all.
extern.C fn exit ['code': i32];
extern.C fn getenv ['name': ptr[u8]] -> ptr[u8];
extern.C fn atoll ['s': ptr[u8]] -> i64[min: 0, max: 200_000_000];

# N, from the text in the environment: missing when there is none, and out
# of range when it is not a number this program takes.
fn read_n ['text': ptr[u8]] -> i64[min: 0, max: 200_000_000] or fail[missing, out_of_range] {
    if ptr.is_null['text'] { fail missing; }
    no_check let 'n' := atoll['text'] otherwise fail;
    return 'n';
}

fn run [] -> i32 or fail[missing, out_of_range] {
    let 'key' := "N";
    no_check let 'n' := read_n[getenv[lend 'key']] otherwise fail;
    # n + 1 falses, as C's calloc gives them (`list.fill`, owner 2026-09-30).
    let mut 'comp': list[bool] = list.fill[value: false, count: 'n' plus 1];
    let mut 'count': i64 = 0;
    for 'p': i64 [from: 2, to: 'n', by: 1] {
        if not 'comp'[pos: to.u64['p' plus 1]] {
            change 'count' = 'count' plus 1;
            for 'm': i64 [from: 'p' times 'p', to: 'n', by: 'p'] {
                change 'comp'[pos: to.u64['m' plus 1]] = true;
            }
        }
    }
    return to.i32['count' mod.floor 256];
}

MAIN {
    let 'code' := run[] otherwise 2;
    no_check my.exit['code'];
}
