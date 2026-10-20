# A bare literal takes its context's family inside a list too (types.md;
# design claude, 0974c85): 1, -2 and 5 among bins are bins, 1 and 2 among
# decis are decimals, and list.fill's 2 fills a list of bins.
# 4 + 3 + 2 + 1.25 x 4 = 14.
extern.C fn exit ['code': i32];
MAIN {
    let 'bs': list[f64] = [1, -2, 5];
    let 'ds': list[d64] = [1, 2];
    let 'fs': list[f64] = list.fill[value: 2, count: 3];
    let 'q': d64 = 1.25;
    let 'qs': list[d64] = list.fill[value: 'q', count: 2];
    let 'b': i32 = to.i32['bs'[pos: 1] plus 'bs'[pos: 2] plus 'bs'[pos: 3]] otherwise 90;
    let 'd': i32 = to.i32['ds'[pos: 1] plus 'ds'[pos: 2]] otherwise 91;
    let 'f': i32 = to.i32['fs'[pos: 3]] otherwise 92;
    let 'e': i32 = to.i32['qs'[pos: 2] times 4] otherwise 93;
    no_check my.exit['b' plus 'd' plus 'f' plus 'e'];
}
