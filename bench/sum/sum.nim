import os, strutils
proc c_exit(code: cint) {.importc: "exit", header: "<stdlib.h>", noreturn.}
let n = try: parseInt(getEnv("N")) except ValueError: -1
if n < 0 or n > 1_000_000_000: quit(2)
var acc = 0
for i in 1..n: acc = (acc + (i * i) mod 7) mod 1_000_000_007
c_exit(cint(acc mod 256))
