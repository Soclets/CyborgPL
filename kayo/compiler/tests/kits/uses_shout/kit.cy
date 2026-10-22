# examples/uses_case as an exit: `shout` is case-insensitive for its fns,
# structs, fields and consts (owner, 2026-10-06), so this Kit reaches them
# in any case, always with the path (owner, 2026-10-22). "hey!" is 4 bytes, "psst, volume 11!"
# 16, the volume 11, and a loud level 100: 131.
extern.C fn exit ['code': i32];
MAIN {
    let 'a' := shout::SHOUT["hey"];
    let 'b' := shout::WHISPER[];
    let 'l' := shout::LEVEL[LOUD = true];
    let mut 'n' := to.i32[std::str.bytes[lend 'a'] plus std::str.bytes[lend 'b']] plus shout::'VOLUME';
    if 'l'.loud { change 'n' = 'n' plus 100; }
    no_check my.exit['n'];
}
