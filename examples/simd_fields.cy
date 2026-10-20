# A simd value goes anywhere a number does (owner, 2026-09-27: a): a struct's field, a list's
# element, a parameter, a return value. It is plain, a fixed-size value that copies by
# itself, and the compiler keeps it on its register-width boundary (16 bytes here).
struct Particle { place: simd[f32, count: 4], vel: simd[f32, count: 4] }
fn step ['p': Particle] -> Particle {
    return Particle[place = 'p'.place + 'p'.vel, vel = 'p'.vel];
}
MAIN {
    let 'p' := Particle[place = simd.fill[value: 0.0], vel = simd.fill[value: 0.5]];
    let 'q' := step[give 'p'];
    let mut 'history': list[simd[f32, count: 4]] = [];
    list.append[lend.mut 'history', value: 'q'.place];
    list.append[lend.mut 'history', value: 'q'.place + 'q'.vel];
    std::print.useful[simd.sum['q'.place], " ", list.length[lend 'history'], " ", simd.sum['history'[pos: 2]], newln];
}
