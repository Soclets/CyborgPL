const std = @import("std");

fn fib(n: i64) i64 {
    return if (n < 2) n else fib(n - 1) + fib(n - 2);
}

pub fn main(init: std.process.Init) void {
    const s = init.environ_map.get("N") orelse std.process.exit(2);
    const n = std.fmt.parseInt(i64, s, 10) catch std.process.exit(2);
    if (n < 0 or n > 92) std.process.exit(2);
    std.process.exit(@intCast(@mod(fib(n), 256)));
}
