# The keys a map can hold are tracked as a set that may be too large, never
# too small: a `set` of a key surely absent adds one entry, any other adds
# none or one. The same key twice into a map bounded to one entry fits.
extern.C fn exit ['code': int32];
MAIN {
    let mut 'm': map[str.utf8, int32][min 0, max 1] = map.empty;
    map.set[lend.mut 'm', key "ab", value 5];
    map.set[lend.mut 'm', key "ab", value 7];
    let 'v' := map.get[lend 'm', key "ab"] otherwise 0;
    no_check my.exit['v' plus to.int32[map.length[lend 'm']]];
}
