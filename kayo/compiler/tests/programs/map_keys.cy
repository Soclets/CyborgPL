extern.C fn exit ['code': i32];
struct Point { x: i32, y: i32 }
MAIN {
    let mut 'big': map[i32, i32] = map.empty;
    for 'i': i32 from 1 to 200 by 1 {
        map.set[lend.mut 'big', key 'i', value 'i' times 2];
    }
    let 'v' := map.get[lend 'big', key 150] otherwise 0;
    let 'gone' := map.remove[lend.mut 'big', key 1] otherwise 0;
    let 'n': i32 = map.length[lend 'big'];
    let mut 'grid': map[Point, i32] = map.empty;
    map.set[lend.mut 'grid', key Point[x = 1, y = 2], value 9];
    let 'p' := map.get[lend 'grid', key Point[x = 1, y = 2]] otherwise 0;
    let 'q' := map.get[lend 'grid', key Point[x = 9, y = 9]] otherwise 5;
    no_check my.exit['v' minus 290 plus 'gone' plus 'n' minus 197 plus 'p' plus 'q'];
}
