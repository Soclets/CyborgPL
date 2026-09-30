import os, strutils
proc c_exit(code: cint) {.importc: "exit", header: "<stdlib.h>", noreturn.}
let n = try: parseInt(getEnv("N")) except ValueError: -1
if n < 0 or n > 100_000_000: quit(2)
var xs: seq[int] = @[]
for i in 1..n: xs.add(i mod 1000)
var acc = 0
for r in 0..<10:
  for j in 0..<xs.len: acc += xs[j]
c_exit(cint(acc mod 256))
