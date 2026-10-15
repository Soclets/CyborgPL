// lists, in parallel the proper Go way: one slice; a goroutine per core fills its part and sums it
// ten times.
package main

import (
	"os"
	"runtime"
	"strconv"
	"sync"
)

func main() {
	n, err := strconv.ParseInt(os.Getenv("N"), 10, 64)
	if err != nil || n < 0 || n > 100000000 {
		os.Exit(2)
	}
	xs := make([]int64, n)
	k := int64(runtime.NumCPU())
	parts := make([]int64, k)
	var wg sync.WaitGroup
	for t := int64(0); t < k; t++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			lo, hi := n*t/k, n*(t+1)/k
			for i := lo; i < hi; i++ {
				xs[i] = (i + 1) % 1000
			}
			var acc int64
			for r := 0; r < 10; r++ {
				for i := lo; i < hi; i++ {
					acc += xs[i]
				}
			}
			parts[t] = acc
		}()
	}
	wg.Wait()
	var acc int64
	for _, p := range parts {
		acc += p
	}
	os.Exit(int(acc % 256))
}
