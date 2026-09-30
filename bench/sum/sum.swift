import Foundation
guard let s = ProcessInfo.processInfo.environment["N"], let n = Int(s), n >= 0, n <= 1_000_000_000 else { exit(2) }
var acc = 0
var i = 1
while i <= n { acc += (i &* i) % 7; i += 1 }
exit(Int32(acc % 1_000_000_007 % 256))
