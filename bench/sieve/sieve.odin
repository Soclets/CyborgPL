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
	n := read_n(200_000_000)
	comp := make([]bool, n + 1)
	count: i64 = 0
	for p: i64 = 2; p <= n; p += 1 {
		if !comp[p] {
			count += 1
			for m := p * p; m <= n; m += p { comp[m] = true }
		}
	}
	delete(comp)
	os.exit(int(count % 256))
}
