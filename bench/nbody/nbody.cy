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
incl.%C.system["stdio.h"];
%extern.C proc exit [$code: i32];
%extern.C proc getenv [$name: ptr[u8]] -> ptr[u8];
%extern.C proc atoll [$s: ptr[u8]] -> i64[min: 0, max: 1_000_000_000];
%extern.C proc sqrt [$x: f64] -> f64;

const $pi: f64 := 3.141592653589793;
const $solar_mass: f64 := 4.0 x $pi x $pi;
const $days_per_year: f64 := 365.24;

rec Body:
    var x: f64;
    var y: f64;
    var z: f64;
    var vx: f64;
    var vy: f64;
    var vz: f64;
    var mass: f64;
;rec Body

# N, from the text in the environment: missing when there is none, and out
# of range when it is not a number this program takes.
proc read_n [$text: ptr[u8]] -> i64[min: 0, max: 1_000_000_000] or fail[missing, out_of_range]:
    if ptr.is_null($text):
        fail missing;
    ;if
    no_check let $n := atoll($text)?;
    ret $n;
;proc read_n

proc advance [lent.mut $bodies: std::ls[Body], $dt: f64]:
    let $n := std::ls.len(lend $bodies);
    for $i: u64 [from: 1, to: $n, by: 1]:
        for $j: u64 [from: $i + 1, to: $n, by: 1]:
            no_check:
                let $dx: f64 := $bodies[pos: $i].x - $bodies[pos: $j].x;
                let $dy: f64 := $bodies[pos: $i].y - $bodies[pos: $j].y;
                let $dz: f64 := $bodies[pos: $i].z - $bodies[pos: $j].z;
                let $distance: f64 := sqrt($dx x $dx + $dy x $dy + $dz x $dz);
                let $mag: f64 := $dt div.near ($distance x $distance x $distance);
                change $bodies[pos: $i].vx to $bodies[pos: $i].vx - $dx x $bodies[pos: $j].mass x $mag;
                change $bodies[pos: $i].vy to $bodies[pos: $i].vy - $dy x $bodies[pos: $j].mass x $mag;
                change $bodies[pos: $i].vz to $bodies[pos: $i].vz - $dz x $bodies[pos: $j].mass x $mag;
                change $bodies[pos: $j].vx to $bodies[pos: $j].vx + $dx x $bodies[pos: $i].mass x $mag;
                change $bodies[pos: $j].vy to $bodies[pos: $j].vy + $dy x $bodies[pos: $i].mass x $mag;
                change $bodies[pos: $j].vz to $bodies[pos: $j].vz + $dz x $bodies[pos: $i].mass x $mag;
            ;no_check
        ;for
    ;for
    for $i: u64 [from: 1, to: $n, by: 1]:
        no_check:
            change $bodies[pos: $i].x to $bodies[pos: $i].x + $dt x $bodies[pos: $i].vx;
            change $bodies[pos: $i].y to $bodies[pos: $i].y + $dt x $bodies[pos: $i].vy;
            change $bodies[pos: $i].z to $bodies[pos: $i].z + $dt x $bodies[pos: $i].vz;
        ;no_check
    ;for
;proc advance

proc energy [lent $bodies: std::ls[Body]] -> f64:
    let $n := std::ls.len(lend $bodies);
    var $e: f64 = 0.0;
    for $i: u64 [from: 1, to: $n, by: 1]:
        no_check change $e to $e + 0.5 x $bodies[pos: $i].mass x ($bodies[pos: $i].vx x $bodies[pos: $i].vx + $bodies[pos: $i].vy x $bodies[pos: $i].vy + $bodies[pos: $i].vz x $bodies[pos: $i].vz);
        for $j: u64 [from: $i + 1, to: $n, by: 1]:
            no_check:
                let $dx: f64 := $bodies[pos: $i].x - $bodies[pos: $j].x;
                let $dy: f64 := $bodies[pos: $i].y - $bodies[pos: $j].y;
                let $dz: f64 := $bodies[pos: $i].z - $bodies[pos: $j].z;
                let $distance: f64 := sqrt($dx x $dx + $dy x $dy + $dz x $dz);
                change $e to $e - (($bodies[pos: $i].mass x $bodies[pos: $j].mass) div.near $distance);
            ;no_check
        ;for
    ;for
    ret $e;
;proc energy

proc offset_momentum [lent.mut $bodies: std::ls[Body]]:
    var $px: f64 = 0.0;
    var $py: f64 = 0.0;
    var $pz: f64 = 0.0;
    for $b in lend $bodies:
        no_check:
            change $px to $px + $b.vx x $b.mass;
            change $py to $py + $b.vy x $b.mass;
            change $pz to $pz + $b.vz x $b.mass;
        ;no_check
    ;for
    no_check:
        change $bodies[pos: 1].vx to (0.0 - $px) div.near $solar_mass;
        change $bodies[pos: 1].vy to (0.0 - $py) div.near $solar_mass;
        change $bodies[pos: 1].vz to (0.0 - $pz) div.near $solar_mass;
    ;no_check
;proc offset_momentum

proc run [] -> i32 or fail[missing, out_of_range]:
    let $key := "N";
    no_check let $n := read_n(getenv(lend $key))?;
    var $bodies: std::ls[Body] = [
        # the Sun
        Body[x = 0.0, y = 0.0, z = 0.0, vx = 0.0, vy = 0.0, vz = 0.0, mass = $solar_mass],
        # Jupiter
        Body[x = 4.84143144246472090, y = -1.16032004402742839, z = -0.103622044471123109,
             vx = 0.00166007664274403694 x $days_per_year, vy = 0.00769901118419740425 x $days_per_year,
             vz = -0.0000690460016972063023 x $days_per_year, mass = 0.000954791938424326609 x $solar_mass],
        # Saturn
        Body[x = 8.34336671824457987, y = 4.12479856412430479, z = -0.403523417114321381,
             vx = -0.00276742510726862411 x $days_per_year, vy = 0.00499852801234917238 x $days_per_year,
             vz = 0.0000230417297573763929 x $days_per_year, mass = 0.000285885980666130812 x $solar_mass],
        # Uranus
        Body[x = 12.8943695621391310, y = -15.1111514016986312, z = -0.223307578892655734,
             vx = 0.00296460137564761618 x $days_per_year, vy = 0.00237847173959480950 x $days_per_year,
             vz = -0.0000296589568540237556 x $days_per_year, mass = 0.0000436624404335156298 x $solar_mass],
        # Neptune
        Body[x = 15.3796971148509165, y = -25.9193146099879641, z = 0.179258772950371181,
             vx = 0.00268067772490389322 x $days_per_year, vy = 0.00162824170038242295 x $days_per_year,
             vz = -0.0000951592254519715870 x $days_per_year, mass = 0.0000515138902046611451 x $solar_mass]
    ];
    offset_momentum(lend.mut $bodies);
    let $fmt := %C.str.make("%.9f" concat newln)-
    otherwise:
        case has_zero_byte:
            ret 2;
        ;case has_zero_byte
    ;otherwise;
    no_check printf(lend $fmt, energy(lend $bodies));
    for $step: i64 [from: 1, to: $n, by: 1]:
        advance(lend.mut $bodies, 0.01);
    ;for
    no_check printf(lend $fmt, energy(lend $bodies));
    ret 0;
;proc run

MAIN:
    let $code := run() ?? 2;
    no_check my.exit($code);
;MAIN
