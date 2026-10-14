extern.C fn exit ['code': int32];
fn produce [lent 'out': channel[list[int32]][min 0, max 4], 'base': int32] {
    for 'i': int32 from 1 to 50 by 1 {
        let 'batch': list[int32] = ['base', 'i'];
        channel.send[lend 'out', value give 'batch'];
    }
}
MAIN {
    let 'pipe' := channel[list[int32]][min 0, max 4];
    let 'quiet' := channel[int32];
    concur with [lend 'pipe'] { produce[lend 'pipe', 1000]; }
    concur with [lend 'pipe'] { produce[lend 'pipe', 2000]; }
    concur with [lend 'quiet'] { channel.send[lend 'quiet', value 7]; }
    let mut 'batches': int32 = 0;
    let mut 'sevens': int32 = 0;
    let mut 'closed': int32 = 0;
    for 'k': int32 from 1 to 102 by 1 {
        whichever {
            'b' from lend 'pipe'  { change 'batches' = 'batches' plus to.int32[list.length[lend 'b']] minus 1; }
            's' from lend 'quiet' { change 'sevens' = 'sevens' plus 's'; }
        } otherwise { closed { change 'closed' = 'closed' plus 1; } };
    }
    no_check my.exit['batches' plus 'sevens' plus 'closed' times 10];
}
