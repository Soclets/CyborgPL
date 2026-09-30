import os, strutils
proc c_exit(code: cint) {.importc: "exit", header: "<stdlib.h>", noreturn.}
let n = try: parseInt(getEnv("N")) except ValueError: -1
if n < 0 or n > 200_000_000: quit(2)
var comp = newSeq[bool](n + 1)
var count = 0
for p in 2..n:
  if not comp[p]:
    inc count
    var m = p * p
    while m <= n:
      comp[m] = true
      m += p
c_exit(cint(count mod 256))
