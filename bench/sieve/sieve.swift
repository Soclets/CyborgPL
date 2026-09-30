import Foundation
guard let s = ProcessInfo.processInfo.environment["N"], let n = Int(s), n >= 0, n <= 200_000_000 else { exit(2) }
var comp = [Bool](repeating: false, count: n + 1)
var count = 0
var p = 2
while p <= n {
    if !comp[p] { count += 1; var m = p * p; while m <= n { comp[m] = true; m += p } }
    p += 1
}
exit(Int32(count % 256))
