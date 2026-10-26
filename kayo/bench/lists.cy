# A list of N numbers (i mod 1000 for i = 1 to N), then summed ten times by
# position; the exit code is the sum's low byte. N comes from the
# environment and must be 0 to 100_000_000; anything else ends the program
# with exit code 2.
%extern.C fn exit [$code: i32];
%extern.C fn getenv [$name: ptr[u8]] -> ptr[u8];
%extern.C fn atoll [$s: ptr[u8]] -> i64[min: 0, max: 100_000_000];

# N, from the text in the environment: missing when there is none, and out
# of range when it is not a number this program takes.
fn read_n [$text: ptr[u8]] -> i64[min: 0, max: 100_000_000] or fail[missing, out_of_range] {
    if ptr.is_null($text) { fail missing; }
    no_check let $n := atoll($text) otherwise fail;
    return $n;
}

fn run [] -> i32 or fail[missing, out_of_range] {
    let $key := "N";
    no_check let $n := read_n(getenv(lend $key)) otherwise fail;
    let mut $xs: std::list[i64] = [];
    let mut $i: i64 = 1;
    while $i at_most $n {
        std::list.append(lend.mut $xs, value: $i mod.floor 1000);
        change $i = $i plus 1;
    }
    let mut $acc: i64 = 0;
    let mut $r: i64 = 0;
    while $r < 10 {
        let mut $j: u64 = 1;
        while $j at_most std::list.length(lend $xs) {
            change $acc = $acc plus $xs[pos: $j];
            change $j = $j plus 1;
        }
        change $r = $r plus 1;
    }
    return to.i32($acc mod.floor 256);
}

MAIN {
    let $code := run() otherwise 2;
    no_check my.exit($code);
}
