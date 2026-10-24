# n-body, after the Computer Language Benchmarks Game: the Sun and the four Jovian planets, N steps of
# the symplectic integrator with dt = 0.01, the energy printed to 9 places before and after. N comes
# from the environment and must be 0 to 1_000_000_000; anything else ends the program with exit code 2,
# as does no N at all. Every version in bench/nbody/ does this arithmetic in this order, so their
# energies agree to the last digit; none fuses a multiply and an add.
#
# std has no square root and no fixed-places print yet (`std::print.format` is not built), so both are
# C's: `sqrt`, and `printf` with "%.9f", the same formatting C's version uses. Every bin operation is a
# check site (an infinity or a NaN), which the prover cannot rule out over N steps, so the arithmetic
# is under `no_check`.
incl.C.system["stdio.h"];
extern.C fn exit [$code: i32];
extern.C fn getenv [$name: ptr[u8]] -> ptr[u8];
extern.C fn atoll [$s: ptr[u8]] -> i64[min: 0, max: 1_000_000_000];
extern.C fn sqrt [$x: f64] -> f64;

const $pi: f64 = 3.141592653589793;
const $solar_mass: f64 = 4.0 times $pi times $pi;
const $days_per_year: f64 = 365.24;

struct Body { x: f64, y: f64, z: f64, vx: f64, vy: f64, vz: f64, mass: f64 }

# N, from the text in the environment: missing when there is none, and out
# of range when it is not a number this program takes.
fn read_n [$text: ptr[u8]] -> i64[min: 0, max: 1_000_000_000] or fail[missing, out_of_range] {
    if ptr.is_null($text) { fail missing; }
    no_check let $n := atoll($text) otherwise fail;
    return $n;
}

fn advance [lent.mut $bodies: std::list[Body], $dt: f64] {
    let $n := std::list.length(lend $bodies);
    for $i: u64 [from: 1, to: $n, by: 1] {
        for $j: u64 [from: $i plus 1, to: $n, by: 1] {
            no_check {
                let $dx: f64 = $bodies[pos: $i].x minus $bodies[pos: $j].x;
                let $dy: f64 = $bodies[pos: $i].y minus $bodies[pos: $j].y;
                let $dz: f64 = $bodies[pos: $i].z minus $bodies[pos: $j].z;
                let $distance: f64 = sqrt($dx times $dx plus $dy times $dy plus $dz times $dz);
                let $mag: f64 = $dt div.near ($distance times $distance times $distance);
                change $bodies[pos: $i].vx = $bodies[pos: $i].vx minus $dx times $bodies[pos: $j].mass times $mag;
                change $bodies[pos: $i].vy = $bodies[pos: $i].vy minus $dy times $bodies[pos: $j].mass times $mag;
                change $bodies[pos: $i].vz = $bodies[pos: $i].vz minus $dz times $bodies[pos: $j].mass times $mag;
                change $bodies[pos: $j].vx = $bodies[pos: $j].vx plus $dx times $bodies[pos: $i].mass times $mag;
                change $bodies[pos: $j].vy = $bodies[pos: $j].vy plus $dy times $bodies[pos: $i].mass times $mag;
                change $bodies[pos: $j].vz = $bodies[pos: $j].vz plus $dz times $bodies[pos: $i].mass times $mag;
            }
        }
    }
    for $i: u64 [from: 1, to: $n, by: 1] {
        no_check {
            change $bodies[pos: $i].x = $bodies[pos: $i].x plus $dt times $bodies[pos: $i].vx;
            change $bodies[pos: $i].y = $bodies[pos: $i].y plus $dt times $bodies[pos: $i].vy;
            change $bodies[pos: $i].z = $bodies[pos: $i].z plus $dt times $bodies[pos: $i].vz;
        }
    }
}

fn energy [lent $bodies: std::list[Body]] -> f64 {
    let $n := std::list.length(lend $bodies);
    let mut $e: f64 = 0.0;
    for $i: u64 [from: 1, to: $n, by: 1] {
        no_check change $e = $e plus 0.5 times $bodies[pos: $i].mass times ($bodies[pos: $i].vx times $bodies[pos: $i].vx plus $bodies[pos: $i].vy times $bodies[pos: $i].vy plus $bodies[pos: $i].vz times $bodies[pos: $i].vz);
        for $j: u64 [from: $i plus 1, to: $n, by: 1] {
            no_check {
                let $dx: f64 = $bodies[pos: $i].x minus $bodies[pos: $j].x;
                let $dy: f64 = $bodies[pos: $i].y minus $bodies[pos: $j].y;
                let $dz: f64 = $bodies[pos: $i].z minus $bodies[pos: $j].z;
                let $distance: f64 = sqrt($dx times $dx plus $dy times $dy plus $dz times $dz);
                change $e = $e minus (($bodies[pos: $i].mass times $bodies[pos: $j].mass) div.near $distance);
            }
        }
    }
    return $e;
}

fn offset_momentum [lent.mut $bodies: std::list[Body]] {
    let mut $px: f64 = 0.0;
    let mut $py: f64 = 0.0;
    let mut $pz: f64 = 0.0;
    for $b in lend $bodies {
        no_check {
            change $px = $px plus $b.vx times $b.mass;
            change $py = $py plus $b.vy times $b.mass;
            change $pz = $pz plus $b.vz times $b.mass;
        }
    }
    no_check {
        change $bodies[pos: 1].vx = (0.0 minus $px) div.near $solar_mass;
        change $bodies[pos: 1].vy = (0.0 minus $py) div.near $solar_mass;
        change $bodies[pos: 1].vz = (0.0 minus $pz) div.near $solar_mass;
    }
}

fn run [] -> i32 or fail[missing, out_of_range] {
    let $key := "N";
    no_check let $n := read_n(getenv(lend $key)) otherwise fail;
    let mut $bodies: std::list[Body] = [
        # the Sun
        Body[x = 0.0, y = 0.0, z = 0.0, vx = 0.0, vy = 0.0, vz = 0.0, mass = $solar_mass],
        # Jupiter
        Body[x = 4.84143144246472090, y = -1.16032004402742839, z = -0.103622044471123109,
             vx = 0.00166007664274403694 times $days_per_year, vy = 0.00769901118419740425 times $days_per_year,
             vz = -0.0000690460016972063023 times $days_per_year, mass = 0.000954791938424326609 times $solar_mass],
        # Saturn
        Body[x = 8.34336671824457987, y = 4.12479856412430479, z = -0.403523417114321381,
             vx = -0.00276742510726862411 times $days_per_year, vy = 0.00499852801234917238 times $days_per_year,
             vz = 0.0000230417297573763929 times $days_per_year, mass = 0.000285885980666130812 times $solar_mass],
        # Uranus
        Body[x = 12.8943695621391310, y = -15.1111514016986312, z = -0.223307578892655734,
             vx = 0.00296460137564761618 times $days_per_year, vy = 0.00237847173959480950 times $days_per_year,
             vz = -0.0000296589568540237556 times $days_per_year, mass = 0.0000436624404335156298 times $solar_mass],
        # Neptune
        Body[x = 15.3796971148509165, y = -25.9193146099879641, z = 0.179258772950371181,
             vx = 0.00268067772490389322 times $days_per_year, vy = 0.00162824170038242295 times $days_per_year,
             vz = -0.0000951592254519715870 times $days_per_year, mass = 0.0000515138902046611451 times $solar_mass]
    ];
    offset_momentum(lend.mut $bodies);
    let $fmt := C.str.make("%.9f" concat newln) otherwise { has_zero_byte { return 2; } };
    no_check printf(lend $fmt, energy(lend $bodies));
    for $step: i64 [from: 1, to: $n, by: 1] {
        advance(lend.mut $bodies, 0.01);
    }
    no_check printf(lend $fmt, energy(lend $bodies));
    return 0;
}

MAIN {
    let $code := run() otherwise 2;
    no_check my.exit($code);
}
