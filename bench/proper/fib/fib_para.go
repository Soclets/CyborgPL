// fib, in parallel the proper Go way: the top of the recursion tree split into goroutines;
// below that, naive recursion as the program asks.
package main

import (
	"os"
	"strconv"
	"sync"
)

func fib(n int64) int64 {
	if n < 2 {
		return n
	}
	return fib(n-1) + fib(n-2)
}

func par(n int64, depth int) int64 {
	if n < 2 || depth == 0 {
		return fib(n)
	}
	var a int64
	var wg sync.WaitGroup
	wg.Add(1)
	go func() { defer wg.Done(); a = par(n-1, depth-1) }()
	b := par(n-2, depth-1)
	wg.Wait()
	return a + b
}

func main() {
	n, err := strconv.ParseInt(os.Getenv("N"), 10, 64)
	if err != nil || n < 0 || n > 92 {
		os.Exit(2)
	}
	os.Exit(int(par(n, 4) % 256))
}
