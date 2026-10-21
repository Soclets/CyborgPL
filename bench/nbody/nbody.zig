// n-body, after the Computer Language Benchmarks Game: the Sun and the four Jovian planets, N steps of
// the symplectic integrator with dt = 0.01, the energy printed to 9 places before and after. N comes
// from the environment and must be 0 to 1_000_000_000; anything else ends the program with exit code 2.
// Every version in bench/nbody/ does this arithmetic in this order, so their energies agree to the last
// digit; none fuses a multiply and an add (Zig's float mode is .strict unless a block sets .optimized).
const std = @import("std");

const pi: f64 = 3.141592653589793;
const solar_mass: f64 = 4.0 * pi * pi;
const days_per_year: f64 = 365.24;

const Body = struct { x: f64, y: f64, z: f64, vx: f64, vy: f64, vz: f64, mass: f64 };

fn advance(bodies: []Body, dt: f64) void {
    for (bodies, 0..) |*b, i| {
        for (bodies[i + 1 ..]) |*b2| {
            const dx = b.x - b2.x;
            const dy = b.y - b2.y;
            const dz = b.z - b2.z;
            const distance = @sqrt(dx * dx + dy * dy + dz * dz);
            const mag = dt / (distance * distance * distance);
            b.vx = b.vx - dx * b2.mass * mag;
            b.vy = b.vy - dy * b2.mass * mag;
            b.vz = b.vz - dz * b2.mass * mag;
            b2.vx = b2.vx + dx * b.mass * mag;
            b2.vy = b2.vy + dy * b.mass * mag;
            b2.vz = b2.vz + dz * b.mass * mag;
        }
    }
    for (bodies) |*b| {
        b.x = b.x + dt * b.vx;
        b.y = b.y + dt * b.vy;
        b.z = b.z + dt * b.vz;
    }
}

fn energy(bodies: []const Body) f64 {
    var e: f64 = 0.0;
    for (bodies, 0..) |*b, i| {
        e = e + 0.5 * b.mass * (b.vx * b.vx + b.vy * b.vy + b.vz * b.vz);
        for (bodies[i + 1 ..]) |*b2| {
            const dx = b.x - b2.x;
            const dy = b.y - b2.y;
            const dz = b.z - b2.z;
            const distance = @sqrt(dx * dx + dy * dy + dz * dz);
            e = e - (b.mass * b2.mass) / distance;
        }
    }
    return e;
}

fn offsetMomentum(bodies: []Body) void {
    var px: f64 = 0.0;
    var py: f64 = 0.0;
    var pz: f64 = 0.0;
    for (bodies) |b| {
        px = px + b.vx * b.mass;
        py = py + b.vy * b.mass;
        pz = pz + b.vz * b.mass;
    }
    bodies[0].vx = -px / solar_mass;
    bodies[0].vy = -py / solar_mass;
    bodies[0].vz = -pz / solar_mass;
}

pub fn main(init: std.process.Init) void {
    const s = init.environ_map.get("N") orelse std.process.exit(2);
    const n = std.fmt.parseInt(i64, s, 10) catch std.process.exit(2);
    if (n < 0 or n > 1_000_000_000) std.process.exit(2);
    var bodies = [5]Body{
        // the Sun
        .{ .x = 0.0, .y = 0.0, .z = 0.0, .vx = 0.0, .vy = 0.0, .vz = 0.0, .mass = solar_mass },
        // Jupiter
        .{
            .x = 4.84143144246472090e+00, .y = -1.16032004402742839e+00, .z = -1.03622044471123109e-01,
            .vx = 1.66007664274403694e-03 * days_per_year, .vy = 7.69901118419740425e-03 * days_per_year,
            .vz = -6.90460016972063023e-05 * days_per_year, .mass = 9.54791938424326609e-04 * solar_mass,
        },
        // Saturn
        .{
            .x = 8.34336671824457987e+00, .y = 4.12479856412430479e+00, .z = -4.03523417114321381e-01,
            .vx = -2.76742510726862411e-03 * days_per_year, .vy = 4.99852801234917238e-03 * days_per_year,
            .vz = 2.30417297573763929e-05 * days_per_year, .mass = 2.85885980666130812e-04 * solar_mass,
        },
        // Uranus
        .{
            .x = 1.28943695621391310e+01, .y = -1.51111514016986312e+01, .z = -2.23307578892655734e-01,
            .vx = 2.96460137564761618e-03 * days_per_year, .vy = 2.37847173959480950e-03 * days_per_year,
            .vz = -2.96589568540237556e-05 * days_per_year, .mass = 4.36624404335156298e-05 * solar_mass,
        },
        // Neptune
        .{
            .x = 1.53796971148509165e+01, .y = -2.59193146099879641e+01, .z = 1.79258772950371181e-01,
            .vx = 2.68067772490389322e-03 * days_per_year, .vy = 1.62824170038242295e-03 * days_per_year,
            .vz = -9.51592254519715870e-05 * days_per_year, .mass = 5.15138902046611451e-05 * solar_mass,
        },
    };
    var buffer: [64]u8 = undefined;
    var stdout_writer = std.Io.File.stdout().writer(init.io, &buffer);
    const out = &stdout_writer.interface;
    offsetMomentum(&bodies);
    out.print("{d:.9}\n", .{energy(&bodies)}) catch std.process.exit(3);
    var i: i64 = 1;
    while (i <= n) : (i += 1) advance(&bodies, 0.01);
    out.print("{d:.9}\n", .{energy(&bodies)}) catch std.process.exit(3);
    out.flush() catch std.process.exit(3);
}
