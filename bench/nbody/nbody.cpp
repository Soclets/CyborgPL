// n-body, after the Computer Language Benchmarks Game: the Sun and the four Jovian planets, N steps of
// the symplectic integrator with dt = 0.01, the energy printed to 9 places before and after. N comes
// from the environment and must be 0 to 1_000_000_000; anything else ends the program with exit code 2.
// Every version in bench/nbody/ does this arithmetic in this order, so their energies agree to the last
// digit; none fuses a multiply and an add (built with -ffp-contract=off).
#include <array>
#include <cmath>
#include <cstdio>
#include <cstdlib>
#include <string>

constexpr double pi = 3.141592653589793;
constexpr double solar_mass = 4.0 * pi * pi;
constexpr double days_per_year = 365.24;

struct Body {
    double x, y, z, vx, vy, vz, mass;
};

using Bodies = std::array<Body, 5>;

static void advance(Bodies &bodies, double dt) {
    for (std::size_t i = 0; i < bodies.size(); i++) {
        Body &b = bodies[i];
        for (std::size_t j = i + 1; j < bodies.size(); j++) {
            Body &b2 = bodies[j];
            double dx = b.x - b2.x;
            double dy = b.y - b2.y;
            double dz = b.z - b2.z;
            double distance = std::sqrt(dx * dx + dy * dy + dz * dz);
            double mag = dt / (distance * distance * distance);
            b.vx = b.vx - dx * b2.mass * mag;
            b.vy = b.vy - dy * b2.mass * mag;
            b.vz = b.vz - dz * b2.mass * mag;
            b2.vx = b2.vx + dx * b.mass * mag;
            b2.vy = b2.vy + dy * b.mass * mag;
            b2.vz = b2.vz + dz * b.mass * mag;
        }
    }
    for (Body &b : bodies) {
        b.x = b.x + dt * b.vx;
        b.y = b.y + dt * b.vy;
        b.z = b.z + dt * b.vz;
    }
}

static double energy(const Bodies &bodies) {
    double e = 0.0;
    for (std::size_t i = 0; i < bodies.size(); i++) {
        const Body &b = bodies[i];
        e = e + 0.5 * b.mass * (b.vx * b.vx + b.vy * b.vy + b.vz * b.vz);
        for (std::size_t j = i + 1; j < bodies.size(); j++) {
            const Body &b2 = bodies[j];
            double dx = b.x - b2.x;
            double dy = b.y - b2.y;
            double dz = b.z - b2.z;
            double distance = std::sqrt(dx * dx + dy * dy + dz * dz);
            e = e - (b.mass * b2.mass) / distance;
        }
    }
    return e;
}

static void offset_momentum(Bodies &bodies) {
    double px = 0.0, py = 0.0, pz = 0.0;
    for (const Body &b : bodies) {
        px = px + b.vx * b.mass;
        py = py + b.vy * b.mass;
        pz = pz + b.vz * b.mass;
    }
    bodies[0].vx = -px / solar_mass;
    bodies[0].vy = -py / solar_mass;
    bodies[0].vz = -pz / solar_mass;
}

int main() {
    const char *s = std::getenv("N");
    long long n = s ? std::atoll(s) : -1;
    if (n < 0 || n > 1000000000) return 2;
    Bodies bodies = {{
        // the Sun
        {0.0, 0.0, 0.0, 0.0, 0.0, 0.0, solar_mass},
        // Jupiter
        {4.84143144246472090e+00, -1.16032004402742839e+00, -1.03622044471123109e-01,
         1.66007664274403694e-03 * days_per_year, 7.69901118419740425e-03 * days_per_year,
         -6.90460016972063023e-05 * days_per_year, 9.54791938424326609e-04 * solar_mass},
        // Saturn
        {8.34336671824457987e+00, 4.12479856412430479e+00, -4.03523417114321381e-01,
         -2.76742510726862411e-03 * days_per_year, 4.99852801234917238e-03 * days_per_year,
         2.30417297573763929e-05 * days_per_year, 2.85885980666130812e-04 * solar_mass},
        // Uranus
        {1.28943695621391310e+01, -1.51111514016986312e+01, -2.23307578892655734e-01,
         2.96460137564761618e-03 * days_per_year, 2.37847173959480950e-03 * days_per_year,
         -2.96589568540237556e-05 * days_per_year, 4.36624404335156298e-05 * solar_mass},
        // Neptune
        {1.53796971148509165e+01, -2.59193146099879641e+01, 1.79258772950371181e-01,
         2.68067772490389322e-03 * days_per_year, 1.62824170038242295e-03 * days_per_year,
         -9.51592254519715870e-05 * days_per_year, 5.15138902046611451e-05 * solar_mass},
    }};
    offset_momentum(bodies);
    std::printf("%.9f\n", energy(bodies));
    for (long long i = 1; i <= n; i++) advance(bodies, 0.01);
    std::printf("%.9f\n", energy(bodies));
    return 0;
}
