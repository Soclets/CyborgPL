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

fib :: proc(n: i64) -> i64 { return n < 2 ? n : fib(n - 1) + fib(n - 2) }

main :: proc() {
	n := read_n(92)
	os.exit(int(fib(n) % 256))
}
