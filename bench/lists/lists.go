package main

import (
	"os"
	"strconv"
)

func main() {
	n, err := strconv.ParseInt(os.Getenv("N"), 10, 64)
	if err != nil || n < 0 || n > 100000000 {
		os.Exit(2)
	}
	var xs []int64
	for i := int64(1); i <= n; i++ {
		xs = append(xs, i%1000)
	}
	var acc int64
	for r := 0; r < 10; r++ {
		for j := 0; j < len(xs); j++ {
			acc += xs[j]
		}
	}
	os.Exit(int(acc % 256))
}
