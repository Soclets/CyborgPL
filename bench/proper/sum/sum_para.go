// sum, in parallel the proper Go way: one goroutine per core, a WaitGroup to join them.
package main

import (
	"os"
	"runtime"
	"strconv"
	"sync"
)

func main() {
	n, err := strconv.ParseInt(os.Getenv("N"), 10, 64)
	if err != nil || n < 0 || n > 1000000000 {
		os.Exit(2)
	}
	k := int64(runtime.NumCPU())
	parts := make([]int64, k)
	var wg sync.WaitGroup
	for t := int64(0); t < k; t++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			var acc int64
			for i := 1 + n*t/k; i <= n*(t+1)/k; i++ {
				acc += i * i % 7
			}
			parts[t] = acc
		}()
	}
	wg.Wait()
	var acc int64
	for _, p := range parts {
		acc += p
	}
	os.Exit(int(acc % 1000000007 % 256))
}
