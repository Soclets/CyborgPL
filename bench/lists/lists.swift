import Foundation
guard let s = ProcessInfo.processInfo.environment["N"], let n = Int(s), n >= 0, n <= 100_000_000 else { exit(2) }
var xs: [Int] = []
var i = 1
while i <= n { xs.append(i % 1000); i += 1 }
var acc = 0
for _ in 0..<10 { for j in 0..<xs.count { acc += xs[j] } }
exit(Int32(acc % 256))
