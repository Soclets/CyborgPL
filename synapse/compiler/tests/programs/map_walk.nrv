extern.C fn exit ['code': int32];
MAIN {
    let mut 'm': map[str.utf8, int32] = map.empty;
    map.set[lend.mut 'm', key "seven", value 7];
    map.set[lend.mut 'm', key "three", value 3];
    map.set[lend.mut 'm', key "five", value 5];
    map.set[lend.mut 'm', key "three", value 9];
    let mut 'acc': int32 = 0;
    for 'v' in map.values[lend 'm'] {
        change 'acc' = ('acc' times 3) plus 'v';
    }
    let mut 'keylen': int32 = 0;
    for 'k' in map.keys[lend 'm'] {
        change 'keylen' = 'keylen' plus str.bytes[lend 'k'];
    }
    no_check my.exit['acc' plus 'keylen'];
}
