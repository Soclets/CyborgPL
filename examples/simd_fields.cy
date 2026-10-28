# A simd value goes anywhere a number does (owner, 2026-09-27: a): a struct's field, a list's
# element, a parameter, a return value. It is plain, a fixed-size value that copies by
# itself, and the compiler keeps it on its register-width boundary (16 bytes here).
rec Particle:
    var place: simd[f32, count: 4];
    var vel: simd[f32, count: 4];
;rec Particle
fn step [let $p: Particle] -> Particle:
    ret Particle[place = $p.place + $p.vel, vel = $p.vel];
;fn step
MAIN:
    let $p := Particle[place = simd.fill(value: 0.0), vel = simd.fill(value: 0.5)];
    let $q := step(give $p);
    var $history: std::ls[simd[f32, count: 4]] = [];
    std::ls.append(lend.mut $history, value: $q.place);
    std::ls.append(lend.mut $history, value: $q.place + $q.vel);
    std::print(simd.sum($q.place), " ", std::ls.len(lend $history), " ", simd.sum($history[pos: 2]), newln);
;MAIN
