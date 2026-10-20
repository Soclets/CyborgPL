# Finding 2026-10-10 (check run, 22 seeds such as 48504, cut down by hand): `change` of a const in an
# untaken branch. A const is never assigned to (owner, 2026-10-10); Synapse refuses it ("'r' is a const
# and cannot be assigned to"). The oracle runs the program: it checks the assignment only where it
# runs (in code that runs it refuses; a shadow of the const it refuses even unrun).
MAIN {
    if false {
        const 'r': i32 = 3;
        change 'r' = 4;
    }
    put_u[1];
}
