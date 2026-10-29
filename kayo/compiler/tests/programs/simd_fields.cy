# A simd value goes anywhere a number does (owner, 2026-09-27: a): a struct's
# field, a list's element, a parameter and a return (examples/simd_fields.cpls
# with i32 lanes, read out as an exit code).
%extern.C proc exit [let $code: i32];

rec Particle:
    var place: simd[i32, count: 4]; var vel: simd[i32, count: 4];
;rec Particle

fn step [let $p: Particle] -> Particle:
    ret Particle[place = $p.place + $p.vel, vel = $p.vel];
;fn step

MAIN:
    let $p := Particle[place = simd.fill(value: 1), vel = simd.fill(value: 2)];
    let $q := step(give $p);
    var $history: std::ls[simd[i32, count: 4]] = [];
    std::ls.append(lend.mut $history, value: $q.place);
    std::ls.append(lend.mut $history, value: $q.place + $q.vel);
    std::ls.append(lend.mut $history, value: simd.shuffle($q.vel, [1, 2, 3, 4]));
    # 12 + 3 x 10 + 20 + 8 = 70
    no_check my.exit(simd.sum($q.place) + to.i32(std::ls.len(lend $history)) x 10 + simd.sum($history[pos: 2]) + simd.sum($history[pos: 3]));
;MAIN
