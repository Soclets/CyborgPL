import Foundation
func fib(_ n: Int) -> Int { n < 2 ? n : fib(n - 1) + fib(n - 2) }
guard let s = ProcessInfo.processInfo.environment["N"], let n = Int(s), n >= 0, n <= 92 else { exit(2) }
exit(Int32(fib(n) % 256))
