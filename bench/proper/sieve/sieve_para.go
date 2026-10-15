// sieve, in parallel the proper Go way: the primes up to √N first, then a goroutine per core
// crosses off and counts its own part of one []bool.
package main

import (
	"os"
	"runtime"
	"strconv"
	"sync"
)

func main() {
	n, err := strconv.ParseInt(os.Getenv("N"), 10, 64)
	if err != nil || n < 0 || n > 200000000 {
		os.Exit(2)
	}
	if n < 2 {
		os.Exit(0)
	}
	r := int64(1)
	for (r+1)*(r+1) <= n {
		r++
	}
	small := make([]bool, r+1)
	var base []int64
	for p := int64(2); p <= r; p++ {
		if !small[p] {
			base = append(base, p)
			for m := p * p; m <= r; m += p {
				small[m] = true
			}
		}
	}
	comp := make([]bool, n+1)
	k := int64(runtime.NumCPU())
	counts := make([]int64, k)
	var wg sync.WaitGroup
	for t := int64(0); t < k; t++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			lo, hi := 2+(n-1)*t/k, 1+(n-1)*(t+1)/k
			for _, q := range base {
				m := (lo + q - 1) / q * q
				if m < q*q {
					m = q * q
				}
				for ; m <= hi; m += q {
					comp[m] = true
				}
			}
			var c int64
			for i := lo; i <= hi; i++ {
				if !comp[i] {
					c++
				}
			}
			counts[t] = c
		}()
	}
	wg.Wait()
	var count int64
	for _, c := range counts {
		count += c
	}
	os.Exit(int(count % 256))
}
