# Two failing tasks in one block: each handler runs where its own `concur` was written
# (found by the compiler session, 2026-09-23). Exits with 1 plus 10 = 11.
extern.C fn exit ['code': int32];
fn judge ['n': int32] -> fail[small] { if 'n' < 10 { fail small; } }
MAIN {
    let mut 'seen': int32 = 0;
    {
        concur { judge[3] otherwise fail; } otherwise { small { change 'seen' = 'seen' + 1; } };
        concur { judge[4] otherwise fail; } otherwise { small { change 'seen' = 'seen' + 10; } };
    }
    no_check my.exit['seen'];   # bare `exit` could be C's or std's (owner, 2026-10-06: a kit's own does not win)
}
