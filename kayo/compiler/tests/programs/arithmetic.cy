# CCA's own test programs, not the language's examples (those are the design
# session's). Each one exits with a number this test knows.
extern.C fn exit ['code': i32];

fn add ['a': i32, 'b': i32] -> i32 { return 'a' plus 'b'; }
fn twice ['n': i32] -> i32 { return 'n' times 2; }

MAIN {
    let 'sum': i32 = add[20, 1];
    no_check my.exit[twice['sum']];       # 42
}
