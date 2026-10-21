package main

import "core:os"
import "core:strconv"

// N from the environment, as the C version reads it: missing or out of range ends with exit code 2
read_n :: proc(hi: i64) -> i64 {
	s, ok := os.lookup_env("N", context.temp_allocator)
	if !ok { os.exit(2) }
	n, nok := strconv.parse_i64(s)
	if !nok || n < 0 || n > hi { os.exit(2) }
	return n
}

main :: proc() {
	n := read_n(100_000_000)
	xs: [dynamic]i64
	for i: i64 = 1; i <= n; i += 1 { append(&xs, i % 1000) }
	acc: i64 = 0
	for r := 0; r < 10; r += 1 { for x in xs { acc += x } }
	delete(xs)
	os.exit(int(acc % 256))
}
