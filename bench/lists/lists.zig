const std = @import("std");

pub fn main(init: std.process.Init) void {
    const s = init.environ_map.get("N") orelse std.process.exit(2);
    const n = std.fmt.parseInt(i64, s, 10) catch std.process.exit(2);
    if (n < 0 or n > 100_000_000) std.process.exit(2);
    var xs: std.ArrayList(i64) = .empty;
    var i: i64 = 1;
    while (i <= n) : (i += 1) xs.append(init.gpa, @mod(i, 1000)) catch std.process.exit(3);
    var acc: i64 = 0;
    var r: usize = 0;
    while (r < 10) : (r += 1) {
        for (xs.items) |x| acc += x;
    }
    std.process.exit(@intCast(@mod(acc, 256)));
}
