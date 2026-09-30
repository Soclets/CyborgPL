import os, strutils
proc c_exit(code: cint) {.importc: "exit", header: "<stdlib.h>", noreturn.}
proc fib(n: int): int = (if n < 2: n else: fib(n - 1) + fib(n - 2))
let n = try: parseInt(getEnv("N")) except ValueError: -1
if n < 0 or n > 92: quit(2)
c_exit(cint(fib(n) mod 256))
