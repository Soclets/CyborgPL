const std = @import("std");

pub fn main(init: std.process.Init) void {
    const s = init.environ_map.get("N") orelse std.process.exit(2);
    const n = std.fmt.parseInt(i64, s, 10) catch std.process.exit(2);
    if (n < 0 or n > 200_000_000) std.process.exit(2);
    const un: usize = @intCast(n);
    const comp = init.gpa.alloc(bool, un + 1) catch std.process.exit(3);
    @memset(comp, false);
    var count: i64 = 0;
    var p: usize = 2;
    while (p <= un) : (p += 1) {
        if (!comp[p]) {
            count += 1;
            var m = p * p;
            while (m <= un) : (m += p) comp[m] = true;
        }
    }
    std.process.exit(@intCast(@mod(count, 256)));
}
