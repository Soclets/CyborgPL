# A value a function built and handed back is given back by whoever holds
# it — and a fresh one sent, appended or set is moved in, never given back
# twice. Both were wrong until 2026-09-29: an answer held in a variable was
# never freed, which only hid the second.
extern.C fn exit ['code': i32];
fn page [lent 'u': str.utf8] -> str.utf8 { return "page of " concat 'u'; }
fn run [] -> i32 {
    let mut 'xs': list[str.utf8] = [];
    let 'a' := "a";
    list.append[lend.mut 'xs', value page[lend 'a']];
    list.append[lend.mut 'xs', value page["b"]];
    let mut 'm': map[str.utf8, str.utf8] = map.empty;
    map.set[lend.mut 'm', key page["k"], value page["v"]];
    return to.i32[list.length[lend 'xs']] plus to.i32[map.length[lend 'm']];
}
MAIN {
    no_check my.exit[run[]];
}
