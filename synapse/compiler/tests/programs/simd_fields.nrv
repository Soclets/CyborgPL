# A simd value goes anywhere a number does (owner, 2026-09-27: a): a struct's
# field, a list's element, a parameter and a return (examples/simd_fields.cpls
# with int32 lanes, read out as an exit code).
extern.C fn exit ['code': int32];

struct Particle { place: simd[int32, 4], vel: simd[int32, 4] }

fn step ['p': Particle] -> Particle {
    return Particle[place = 'p'.place plus 'p'.vel, vel = 'p'.vel];
}

MAIN {
    let 'p' := Particle[place = simd.fill[value 1], vel = simd.fill[value 2]];
    let 'q' := step[give 'p'];
    let mut 'history': list[simd[int32, 4]] = [];
    list.append[lend.mut 'history', value 'q'.place];
    list.append[lend.mut 'history', value 'q'.place plus 'q'.vel];
    list.append[lend.mut 'history', value simd.shuffle['q'.vel, [1, 2, 3, 4]]];
    # 12 + 3 x 10 + 20 + 8 = 70
    no_check my.exit[simd.sum['q'.place] plus to.int32[list.length[lend 'history']] times 10 plus simd.sum['history'[pos 2]] plus simd.sum['history'[pos 3]]];
}
