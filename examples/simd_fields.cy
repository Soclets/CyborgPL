# A simd value goes anywhere a number does (owner, 2026-09-27: a): a struct's field, a list's
# element, a parameter, a return value. It is plain, a fixed-size value that copies by
# itself, and the compiler keeps it on its register-width boundary (16 bytes here).
record Particle:
    place: simd[float.binary.32_bit, count: 4]; vel: simd[float.binary.32_bit, count: 4];
end of record Particle
function step [$p: Particle] -> Particle:
    return Particle[place = $p.place plus $p.vel, vel = $p.vel];
end of function step
!MAIN:
    let $p := Particle[place = simd.fill(value: 0.0), vel = simd.fill(value: 0.5)];
    let $q := step(give $p);
    variable $history: std::list[simd[float.binary.32_bit, count: 4]] = [];
    std::list.append(lend.mutable $history, value: $q.place);
    std::list.append(lend.mutable $history, value: $q.place plus $q.vel);
    std::print.shortest(simd.sum($q.place), " ", std::list.length(lend $history), " ", simd.sum($history[position: 2]), newline);
!end of MAIN
