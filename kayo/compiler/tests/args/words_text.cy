# Each word's bytes collected and read as UTF-8 once (owner, 2026-09-30),
# so a word past ASCII arrives whole: run with "héllo", "ab" and the one
# byte 0xFF. Points 5 + 2 = 7, bytes 6 + 2 = 8, and 0xFF is not UTF-8
# (100): 7 x 10 + 8 + 100 = 178.
extern.C fn exit ['code': i32];
MAIN ['argc': i32, 'argv': ptr[ptr[u8]]] {
    let mut 'points': u64 = 0;
    let mut 'bytes': u64 = 0;
    let mut 'bad': i32 = 0;
    let mut 'i': i32 = 1;
    while 'i' < 'argc' {
        no_check let 'word' := ptr.read[ptr.at['argv', by: 'i']];
        let mut 'k': i32 = 0;
        let mut 'raw': std::list[u8] = [];
        loop {
            no_check let 'b' := ptr.read[ptr.at['word', by: 'k']];
            if 'b' equal_to 0 { break; }
            std::list.append[lend.mut 'raw', value: 'b'];
            change 'k' = 'k' plus 1;
        }
        let 'text' := to.str.utf8['raw'] otherwise str.empty;
        if std::str.bytes[lend 'text'] equal_to 0 { change 'bad' = 'bad' plus 100; }
        change 'points' = 'points' plus std::str.points[lend 'text'];
        change 'bytes' = 'bytes' plus std::str.bytes[lend 'text'];
        change 'i' = 'i' plus 1;
    }
    no_check my.exit[to.i32['points' times 10 plus 'bytes'] plus 'bad'];
}
