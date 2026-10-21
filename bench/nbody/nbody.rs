// n-body, after the Computer Language Benchmarks Game: the Sun and the four Jovian planets, N steps of
// the symplectic integrator with dt = 0.01, the energy printed to 9 places before and after. N comes
// from the environment and must be 0 to 1_000_000_000; anything else ends the program with exit code 2.
// Every version in bench/nbody/ does this arithmetic in this order, so their energies agree to the last
// digit; none fuses a multiply and an add (rustc never does unless asked, with `mul_add`).

const PI: f64 = 3.141592653589793;
const SOLAR_MASS: f64 = 4.0 * PI * PI;
const DAYS_PER_YEAR: f64 = 365.24;
const N_BODIES: usize = 5;

#[derive(Clone, Copy)]
struct Body {
    x: f64,
    y: f64,
    z: f64,
    vx: f64,
    vy: f64,
    vz: f64,
    mass: f64,
}

fn advance(bodies: &mut [Body; N_BODIES], dt: f64) {
    for i in 0..N_BODIES {
        let (left, right) = bodies.split_at_mut(i + 1);
        let b = &mut left[i];
        for b2 in right.iter_mut() {
            let dx = b.x - b2.x;
            let dy = b.y - b2.y;
            let dz = b.z - b2.z;
            let distance = (dx * dx + dy * dy + dz * dz).sqrt();
            let mag = dt / (distance * distance * distance);
            b.vx = b.vx - dx * b2.mass * mag;
            b.vy = b.vy - dy * b2.mass * mag;
            b.vz = b.vz - dz * b2.mass * mag;
            b2.vx = b2.vx + dx * b.mass * mag;
            b2.vy = b2.vy + dy * b.mass * mag;
            b2.vz = b2.vz + dz * b.mass * mag;
        }
    }
    for b in bodies.iter_mut() {
        b.x = b.x + dt * b.vx;
        b.y = b.y + dt * b.vy;
        b.z = b.z + dt * b.vz;
    }
}

fn energy(bodies: &[Body; N_BODIES]) -> f64 {
    let mut e = 0.0;
    for i in 0..N_BODIES {
        let b = &bodies[i];
        e = e + 0.5 * b.mass * (b.vx * b.vx + b.vy * b.vy + b.vz * b.vz);
        for b2 in &bodies[i + 1..] {
            let dx = b.x - b2.x;
            let dy = b.y - b2.y;
            let dz = b.z - b2.z;
            let distance = (dx * dx + dy * dy + dz * dz).sqrt();
            e = e - (b.mass * b2.mass) / distance;
        }
    }
    e
}

fn offset_momentum(bodies: &mut [Body; N_BODIES]) {
    let (mut px, mut py, mut pz) = (0.0, 0.0, 0.0);
    for b in bodies.iter() {
        px = px + b.vx * b.mass;
        py = py + b.vy * b.mass;
        pz = pz + b.vz * b.mass;
    }
    bodies[0].vx = -px / SOLAR_MASS;
    bodies[0].vy = -py / SOLAR_MASS;
    bodies[0].vz = -pz / SOLAR_MASS;
}

fn main() {
    let n: i64 = match std::env::var("N").ok().and_then(|s| s.parse().ok()) { Some(v) if v >= 0 && v <= 1_000_000_000 => v, _ => std::process::exit(2) };
    let mut bodies = [
        // the Sun
        Body { x: 0.0, y: 0.0, z: 0.0, vx: 0.0, vy: 0.0, vz: 0.0, mass: SOLAR_MASS },
        // Jupiter
        Body {
            x: 4.84143144246472090e+00, y: -1.16032004402742839e+00, z: -1.03622044471123109e-01,
            vx: 1.66007664274403694e-03 * DAYS_PER_YEAR, vy: 7.69901118419740425e-03 * DAYS_PER_YEAR,
            vz: -6.90460016972063023e-05 * DAYS_PER_YEAR, mass: 9.54791938424326609e-04 * SOLAR_MASS,
        },
        // Saturn
        Body {
            x: 8.34336671824457987e+00, y: 4.12479856412430479e+00, z: -4.03523417114321381e-01,
            vx: -2.76742510726862411e-03 * DAYS_PER_YEAR, vy: 4.99852801234917238e-03 * DAYS_PER_YEAR,
            vz: 2.30417297573763929e-05 * DAYS_PER_YEAR, mass: 2.85885980666130812e-04 * SOLAR_MASS,
        },
        // Uranus
        Body {
            x: 1.28943695621391310e+01, y: -1.51111514016986312e+01, z: -2.23307578892655734e-01,
            vx: 2.96460137564761618e-03 * DAYS_PER_YEAR, vy: 2.37847173959480950e-03 * DAYS_PER_YEAR,
            vz: -2.96589568540237556e-05 * DAYS_PER_YEAR, mass: 4.36624404335156298e-05 * SOLAR_MASS,
        },
        // Neptune
        Body {
            x: 1.53796971148509165e+01, y: -2.59193146099879641e+01, z: 1.79258772950371181e-01,
            vx: 2.68067772490389322e-03 * DAYS_PER_YEAR, vy: 1.62824170038242295e-03 * DAYS_PER_YEAR,
            vz: -9.51592254519715870e-05 * DAYS_PER_YEAR, mass: 5.15138902046611451e-05 * SOLAR_MASS,
        },
    ];
    offset_momentum(&mut bodies);
    println!("{:.9}", energy(&bodies));
    for _ in 0..n {
        advance(&mut bodies, 0.01);
    }
    println!("{:.9}", energy(&bodies));
}
