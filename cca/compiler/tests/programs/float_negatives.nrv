# A negative literal in a bin16, bin32 or brain16 place is written at that
# width, as a positive one is: a brain16 -0.3 was stored as a bin64 into its
# two bytes, so every negative brain16 went wrong (check claude, 13ca93c).
extern.C fn exit ['code': int32];
MAIN {
    let 'a': brain16 = -0.3;
    let 'b': brain16 = -1.75;
    let 'c': bin32 = -0.5;
    let 'h': bin16 = -2.5;
    let 'n1' := to.int64['a' times 10.0] otherwise -100;
    let 'n2' := to.int64[('b' plus 'b') times 2.0] otherwise -100;
    let 'n3' := to.int64['c' times 4.0] otherwise -100;
    let 'n4' := to.int64['h' times 2.0] otherwise -100;
    let 'ta' := to.useful.str.utf8['a'];
    let 'tb' := to.stored.str.utf8['b'];
    let 't' := to.int64[str.bytes[lend 'ta'] plus str.bytes[lend 'tb']];
    no_check my.exit[to.int32['n1' plus 'n2' plus 'n3' plus 'n4' plus 100 plus 't']];
}
