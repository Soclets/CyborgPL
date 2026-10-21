/* n-body, after the Computer Language Benchmarks Game: the Sun and the four Jovian planets, N steps of
   the symplectic integrator with dt = 0.01, the energy printed to 9 places before and after. N comes
   from the environment and must be 0 to 1_000_000_000; anything else ends the program with exit code 2.
   Every version in bench/nbody/ does this arithmetic in this order, so their energies agree to the last
   digit; none fuses a multiply and an add (built with -ffp-contract=off). */
#include <math.h>
#include <stdio.h>
#include <stdlib.h>

#define PI 3.141592653589793
#define SOLAR_MASS (4.0 * PI * PI)
#define DAYS_PER_YEAR 365.24
#define NBODIES 5

struct body { double x, y, z, vx, vy, vz, mass; };

static void advance(struct body *bodies, double dt) {
    for (int i = 0; i < NBODIES; i++) {
        struct body *b = &bodies[i];
        for (int j = i + 1; j < NBODIES; j++) {
            struct body *b2 = &bodies[j];
            double dx = b->x - b2->x;
            double dy = b->y - b2->y;
            double dz = b->z - b2->z;
            double distance = sqrt(dx * dx + dy * dy + dz * dz);
            double mag = dt / (distance * distance * distance);
            b->vx = b->vx - dx * b2->mass * mag;
            b->vy = b->vy - dy * b2->mass * mag;
            b->vz = b->vz - dz * b2->mass * mag;
            b2->vx = b2->vx + dx * b->mass * mag;
            b2->vy = b2->vy + dy * b->mass * mag;
            b2->vz = b2->vz + dz * b->mass * mag;
        }
    }
    for (int i = 0; i < NBODIES; i++) {
        struct body *b = &bodies[i];
        b->x = b->x + dt * b->vx;
        b->y = b->y + dt * b->vy;
        b->z = b->z + dt * b->vz;
    }
}

static double energy(const struct body *bodies) {
    double e = 0.0;
    for (int i = 0; i < NBODIES; i++) {
        const struct body *b = &bodies[i];
        e = e + 0.5 * b->mass * (b->vx * b->vx + b->vy * b->vy + b->vz * b->vz);
        for (int j = i + 1; j < NBODIES; j++) {
            const struct body *b2 = &bodies[j];
            double dx = b->x - b2->x;
            double dy = b->y - b2->y;
            double dz = b->z - b2->z;
            double distance = sqrt(dx * dx + dy * dy + dz * dz);
            e = e - (b->mass * b2->mass) / distance;
        }
    }
    return e;
}

static void offset_momentum(struct body *bodies) {
    double px = 0.0, py = 0.0, pz = 0.0;
    for (int i = 0; i < NBODIES; i++) {
        px = px + bodies[i].vx * bodies[i].mass;
        py = py + bodies[i].vy * bodies[i].mass;
        pz = pz + bodies[i].vz * bodies[i].mass;
    }
    bodies[0].vx = -px / SOLAR_MASS;
    bodies[0].vy = -py / SOLAR_MASS;
    bodies[0].vz = -pz / SOLAR_MASS;
}

int main(void) {
    const char *s = getenv("N");
    long long n = s ? atoll(s) : -1;
    if (n < 0 || n > 1000000000) return 2;
    struct body bodies[NBODIES] = {
        /* the Sun */
        {0.0, 0.0, 0.0, 0.0, 0.0, 0.0, SOLAR_MASS},
        /* Jupiter */
        {4.84143144246472090e+00, -1.16032004402742839e+00, -1.03622044471123109e-01,
         1.66007664274403694e-03 * DAYS_PER_YEAR, 7.69901118419740425e-03 * DAYS_PER_YEAR,
         -6.90460016972063023e-05 * DAYS_PER_YEAR, 9.54791938424326609e-04 * SOLAR_MASS},
        /* Saturn */
        {8.34336671824457987e+00, 4.12479856412430479e+00, -4.03523417114321381e-01,
         -2.76742510726862411e-03 * DAYS_PER_YEAR, 4.99852801234917238e-03 * DAYS_PER_YEAR,
         2.30417297573763929e-05 * DAYS_PER_YEAR, 2.85885980666130812e-04 * SOLAR_MASS},
        /* Uranus */
        {1.28943695621391310e+01, -1.51111514016986312e+01, -2.23307578892655734e-01,
         2.96460137564761618e-03 * DAYS_PER_YEAR, 2.37847173959480950e-03 * DAYS_PER_YEAR,
         -2.96589568540237556e-05 * DAYS_PER_YEAR, 4.36624404335156298e-05 * SOLAR_MASS},
        /* Neptune */
        {1.53796971148509165e+01, -2.59193146099879641e+01, 1.79258772950371181e-01,
         2.68067772490389322e-03 * DAYS_PER_YEAR, 1.62824170038242295e-03 * DAYS_PER_YEAR,
         -9.51592254519715870e-05 * DAYS_PER_YEAR, 5.15138902046611451e-05 * SOLAR_MASS},
    };
    offset_momentum(bodies);
    printf("%.9f\n", energy(bodies));
    for (long long i = 1; i <= n; i++) advance(bodies, 0.01);
    printf("%.9f\n", energy(bodies));
    return 0;
}
