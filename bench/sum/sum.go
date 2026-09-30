package main

import (
	"os"
	"strconv"
)

func main() {
	n, err := strconv.ParseInt(os.Getenv("N"), 10, 64)
	if err != nil || n < 0 || n > 1000000000 {
		os.Exit(2)
	}
	var acc int64
	for i := int64(1); i <= n; i++ {
		acc += (i * i) % 7
	}
	os.Exit(int(acc % 1000000007 % 256))
}
