// sieve, the proper Go way: a []bool, crossing off from p*p.
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
	for p := int64(2); p*p <= n; p++ {
		if !comp[p] {
			for m := p * p; m <= n; m += p {
				comp[m] = true
			}
		}
	}
	count := 0
	for i := int64(2); i <= n; i++ {
		if !comp[i] {
			count++
		}
	}
	os.Exit(count % 256)
}
