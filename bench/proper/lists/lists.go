// lists, the proper Go way: a slice made with its full capacity, then append.
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
	xs := make([]int64, 0, n)
	for i := int64(1); i <= n; i++ {
		xs = append(xs, i%1000)
	}
	var acc int64
	for r := 0; r < 10; r++ {
		for _, x := range xs {
			acc += x
		}
	}
	os.Exit(int(acc % 256))
}
