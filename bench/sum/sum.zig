const std = @import("std");

pub fn main(init: std.process.Init) void {
    const s = init.environ_map.get("N") orelse std.process.exit(2);
    const n = std.fmt.parseInt(i64, s, 10) catch std.process.exit(2);
    if (n < 0 or n > 1_000_000_000) std.process.exit(2);
    var acc: i64 = 0;
    var i: i64 = 1;
    while (i <= n) : (i += 1) acc = @mod(acc + @mod(i * i, 7), 1_000_000_007);
    std.process.exit(@intCast(@mod(acc, 256)));
}
