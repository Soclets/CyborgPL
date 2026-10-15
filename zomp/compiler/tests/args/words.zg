# C's arguments (owner, 2026-09-29): run as `cca run words.cpls -- apples 3
# "hello world"`. argc is 4 (the program's name first); the first word's
# first byte is `a`, 97; the last word is 11 bytes long; argv[argc] is
# null. 4 + 97 + 11 + 100 = 212.
extern.C fn exit ['code': int32];
MAIN ['argc': int32, 'argv': ptr[ptr[uint8]]] {
    no_check let 'first' := ptr.read[ptr.at['argv', by 1]];
    no_check let 'a' := ptr.read['first'];
    no_check let 'last' := ptr.read[ptr.at['argv', by 'argc' minus 1]];
    let mut 'n': int32 = 0;
    loop {
        no_check let 'b' := ptr.read[ptr.at['last', by 'n']];
        if 'b' equal_to 0 { break; }
        change 'n' = 'n' plus 1;
    }
    no_check let 'end' := ptr.read[ptr.at['argv', by 'argc']];
    let mut 'null': int32 = 0;
    if ptr.is_null['end'] { change 'null' = 100; }
    no_check my.exit['argc' plus to.int32['a'] plus 'n' plus 'null'];
}
