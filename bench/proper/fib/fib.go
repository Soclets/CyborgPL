// fib: naive recursion, as the program asks.
package main

import (
	"os"
	"strconv"
)

func fib(n int64) int64 {
	if n < 2 {
		return n
	}
	return fib(n-1) + fib(n-2)
}

func main() {
	n, err := strconv.ParseInt(os.Getenv("N"), 10, 64)
	if err != nil || n < 0 || n > 92 {
		os.Exit(2)
	}
	os.Exit(int(fib(n) % 256))
}
