# A character literal of several code points can only be a grapheme
# (types.md), with its unit unwritten too: e + a combining accent.
extern.C fn exit ['code': i32];
MAIN {
    let 'g' := `é`;
    let 't' := to.str.utf8['g'];
    let mut 'got': i32 = 0;
    if std::str.bytes[lend 't'] equal_to 3 and std::str.graphemes[lend 't'] equal_to 1 { change 'got' = 'got' plus 1; }
    if 'g' equal_to `é` { change 'got' = 'got' plus 2; }
    no_check my.exit['got'];
}
