# A parallel sum over a list after a loop that wrote it (check claude,
# 2026-10-19): Precog knew the list's length, folded the sum loop's stop
# to it, and each slice ran from its own first trip to the loop's end,
# 8000 ones summing to 16001. 8000 ones, then 4538 trues of 20000
# flags: 12538, and 12538 mod 256 is 250.
extern.C fn exit ['code': i32];
MAIN {
    let mut 'b': list[u8] = list.fill[value 0, count 8000];
    for 'i': u64 from 1 to 8000 by 1 { change 'b'[pos 'i'] = 1; }
    let mut 'c': u64 = 0;
    for 'i': u64 from 1 to list.length[lend 'b'] by 1 { change 'c' = 'c' + to.u64['b'[pos 'i']]; }
    let mut 'f': list[bool] = list.fill[value false, count 20000];
    for 'i': u64 from 1 to 4538 by 1 { change 'f'[pos 'i' times 4] = true; }
    for 'i': u64 from 1 to list.length[lend 'f'] by 1 {
        if 'f'[pos 'i'] { change 'c' = 'c' + 1; }
    }
    no_check my.exit[to.i32['c' mod.floor 256]];
}
