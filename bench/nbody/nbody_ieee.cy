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
#
# nbody.cy with every bin operation in its `.ieee` form, which keeps IEEE's -0 as the other five
# languages do, so kayo adds no +0.0 after each one (owner, 2026-10-21: "add the .ieee row").
include.%C.system["stdio.h"];
%external.C procedure exit [$code: integer.signed.32_bit];
%external.C procedure getenv [$name: pointer[integer.unsigned.8_bit]] -> pointer[integer.unsigned.8_bit];
%external.C procedure atoll [$s: pointer[integer.unsigned.8_bit]] -> integer.signed.64_bit[minimum: 0, maximum: 1_000_000_000];
%external.C procedure sqrt [$x: float.binary.64_bit] -> float.binary.64_bit;

constant $pi: float.binary.64_bit = 3.141592653589793;
constant $solar_mass: float.binary.64_bit = 4.0 times.ieee $pi times.ieee $pi;
constant $days_per_year: float.binary.64_bit = 365.24;

record Body:
    x: float.binary.64_bit; y: float.binary.64_bit; z: float.binary.64_bit; vx: float.binary.64_bit; vy: float.binary.64_bit; vz: float.binary.64_bit; mass: float.binary.64_bit;
end of record Body

# N, from the text in the environment: missing when there is none, and out
# of range when it is not a number this program takes.
procedure read_n [$text: pointer[integer.unsigned.8_bit]] -> integer.signed.64_bit[minimum: 0, maximum: 1_000_000_000] or fail[missing, out_of_range]:
    if pointer.is_null($text):
        fail missing;
    end of if
    !no_check let $n := atoll($text) otherwise fail;
    return $n;
end of procedure read_n

procedure advance [lent.mutable $bodies: std::list[Body], $dt: float.binary.64_bit]:
    let $n := std::list.length(lend $bodies);
    for $i: integer.unsigned.64_bit [from: 1, to: $n, by: 1]:
        for $j: integer.unsigned.64_bit [from: $i plus 1, to: $n, by: 1]:
            !no_check:
                let $dx: float.binary.64_bit = $bodies[position: $i].x minus.ieee $bodies[position: $j].x;
                let $dy: float.binary.64_bit = $bodies[position: $i].y minus.ieee $bodies[position: $j].y;
                let $dz: float.binary.64_bit = $bodies[position: $i].z minus.ieee $bodies[position: $j].z;
                let $distance: float.binary.64_bit = sqrt($dx times.ieee $dx plus.ieee $dy times.ieee $dy plus.ieee $dz times.ieee $dz);
                let $mag: float.binary.64_bit = $dt divide.nearest.ieee ($distance times.ieee $distance times.ieee $distance);
                change $bodies[position: $i].vx = $bodies[position: $i].vx minus.ieee $dx times.ieee $bodies[position: $j].mass times.ieee $mag;
                change $bodies[position: $i].vy = $bodies[position: $i].vy minus.ieee $dy times.ieee $bodies[position: $j].mass times.ieee $mag;
                change $bodies[position: $i].vz = $bodies[position: $i].vz minus.ieee $dz times.ieee $bodies[position: $j].mass times.ieee $mag;
                change $bodies[position: $j].vx = $bodies[position: $j].vx plus.ieee $dx times.ieee $bodies[position: $i].mass times.ieee $mag;
                change $bodies[position: $j].vy = $bodies[position: $j].vy plus.ieee $dy times.ieee $bodies[position: $i].mass times.ieee $mag;
                change $bodies[position: $j].vz = $bodies[position: $j].vz plus.ieee $dz times.ieee $bodies[position: $i].mass times.ieee $mag;
            !end of no_check
        end of for
    end of for
    for $i: integer.unsigned.64_bit [from: 1, to: $n, by: 1]:
        !no_check:
            change $bodies[position: $i].x = $bodies[position: $i].x plus.ieee $dt times.ieee $bodies[position: $i].vx;
            change $bodies[position: $i].y = $bodies[position: $i].y plus.ieee $dt times.ieee $bodies[position: $i].vy;
            change $bodies[position: $i].z = $bodies[position: $i].z plus.ieee $dt times.ieee $bodies[position: $i].vz;
        !end of no_check
    end of for
end of procedure advance

function energy [lent $bodies: std::list[Body]] -> float.binary.64_bit:
    let $n := std::list.length(lend $bodies);
    variable $e: float.binary.64_bit = 0.0;
    for $i: integer.unsigned.64_bit [from: 1, to: $n, by: 1]:
        !no_check change $e = $e plus.ieee 0.5 times.ieee $bodies[position: $i].mass times.ieee ($bodies[position: $i].vx times.ieee $bodies[position: $i].vx plus.ieee $bodies[position: $i].vy times.ieee $bodies[position: $i].vy plus.ieee $bodies[position: $i].vz times.ieee $bodies[position: $i].vz);
        for $j: integer.unsigned.64_bit [from: $i plus 1, to: $n, by: 1]:
            !no_check:
                let $dx: float.binary.64_bit = $bodies[position: $i].x minus.ieee $bodies[position: $j].x;
                let $dy: float.binary.64_bit = $bodies[position: $i].y minus.ieee $bodies[position: $j].y;
                let $dz: float.binary.64_bit = $bodies[position: $i].z minus.ieee $bodies[position: $j].z;
                let $distance: float.binary.64_bit = sqrt($dx times.ieee $dx plus.ieee $dy times.ieee $dy plus.ieee $dz times.ieee $dz);
                change $e = $e minus.ieee (($bodies[position: $i].mass times.ieee $bodies[position: $j].mass) divide.nearest.ieee $distance);
            !end of no_check
        end of for
    end of for
    return $e;
end of function energy

procedure offset_momentum [lent.mutable $bodies: std::list[Body]]:
    variable $px: float.binary.64_bit = 0.0;
    variable $py: float.binary.64_bit = 0.0;
    variable $pz: float.binary.64_bit = 0.0;
    for $b in lend $bodies:
        !no_check:
            change $px = $px plus.ieee $b.vx times.ieee $b.mass;
            change $py = $py plus.ieee $b.vy times.ieee $b.mass;
            change $pz = $pz plus.ieee $b.vz times.ieee $b.mass;
        !end of no_check
    end of for
    !no_check:
        change $bodies[position: 1].vx = (0.0 minus.ieee $px) divide.nearest.ieee $solar_mass;
        change $bodies[position: 1].vy = (0.0 minus.ieee $py) divide.nearest.ieee $solar_mass;
        change $bodies[position: 1].vz = (0.0 minus.ieee $pz) divide.nearest.ieee $solar_mass;
    !end of no_check
end of procedure offset_momentum

function run [] -> integer.signed.32_bit or fail[missing, out_of_range]:
    let $key := "N";
    !no_check let $n := read_n(getenv(lend $key)) otherwise fail;
    variable $bodies: std::list[Body] = [
        # the Sun
        Body[x = 0.0, y = 0.0, z = 0.0, vx = 0.0, vy = 0.0, vz = 0.0, mass = $solar_mass],
        # Jupiter
        Body[x = 4.84143144246472090, y = negative 1.16032004402742839, z = negative 0.103622044471123109,
             vx = 0.00166007664274403694 times.ieee $days_per_year, vy = 0.00769901118419740425 times.ieee $days_per_year,
             vz = negative 0.0000690460016972063023 times.ieee $days_per_year, mass = 0.000954791938424326609 times.ieee $solar_mass],
        # Saturn
        Body[x = 8.34336671824457987, y = 4.12479856412430479, z = negative 0.403523417114321381,
             vx = negative 0.00276742510726862411 times.ieee $days_per_year, vy = 0.00499852801234917238 times.ieee $days_per_year,
             vz = 0.0000230417297573763929 times.ieee $days_per_year, mass = 0.000285885980666130812 times.ieee $solar_mass],
        # Uranus
        Body[x = 12.8943695621391310, y = negative 15.1111514016986312, z = negative 0.223307578892655734,
             vx = 0.00296460137564761618 times.ieee $days_per_year, vy = 0.00237847173959480950 times.ieee $days_per_year,
             vz = negative 0.0000296589568540237556 times.ieee $days_per_year, mass = 0.0000436624404335156298 times.ieee $solar_mass],
        # Neptune
        Body[x = 15.3796971148509165, y = negative 25.9193146099879641, z = 0.179258772950371181,
             vx = 0.00268067772490389322 times.ieee $days_per_year, vy = 0.00162824170038242295 times.ieee $days_per_year,
             vz = negative 0.0000951592254519715870 times.ieee $days_per_year, mass = 0.0000515138902046611451 times.ieee $solar_mass]
    ];
    offset_momentum(lend.mutable $bodies);
    let $fmt := %C.string.make("%.9f" concatenate newline) otherwise:
        !case has_zero_byte:
            return 2;
        !end of case has_zero_byte
    end of otherwise;
    !no_check printf(lend $fmt, energy(lend $bodies));
    for $step: integer.signed.64_bit [from: 1, to: $n, by: 1]:
        advance(lend.mutable $bodies, 0.01);
    end of for
    !no_check printf(lend $fmt, energy(lend $bodies));
    return 0;
end of function run

!MAIN:
    let $code := run() otherwise 2;
    !no_check my.exit($code);
!end of MAIN
