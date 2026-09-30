package main

import (
	"os"
	"strconv"
)

func main() {
	n, err := strconv.ParseInt(os.Getenv("N"), 10, 64)
	if err != nil || n < 0 || n > 200000000 {
		os.Exit(2)
	}
	comp := make([]bool, n+1)
	var count int64
	for p := int64(2); p <= n; p++ {
		if !comp[p] {
			count++
			for m := p * p; m <= n; m += p {
				comp[m] = true
			}
		}
	}
	os.Exit(int(count % 256))
}
