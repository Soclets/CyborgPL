# A list of `layout.C` structs bound to a kernel, kayo writing the struct's Metal declaration at the top of the block,
# and shared code from a `.metal` file of the kit, `#include`d from the kit's root (owner, 2026-10-22).
%layout.C struct Body { x: f32, vx: f32 }

MAIN {
    let mut $bodies: std::list[Body] = [Body[x = 1.0, vx = 2.0], Body[x = 0.0, vx = -1.0]];
    let $dt: f32 = 0.5;
    no_check inline.%MSL threads std::list.length(lend $bodies) {
        #include "shaders/common.metal"
        kernel void step(device Body *bodies [[buffer(0)]], constant float &dt [[buffer(1)]], uint id [[thread_position_in_grid]]) {
            bodies[id].x = moved(bodies[id].x, bodies[id].vx, dt);
        }
    } with [lend.mut $bodies -> buffer(0), copy $dt -> buffer(1)];
    for $b in lend $bodies { std::print.useful($b.x, newln); }
}
