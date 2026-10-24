# An unproven site keeps its check in a working build, and a check that
# fires stops the program with 101 and says where.
extern.C fn exit [$code: i32];
extern.C fn getpid [] -> i32;

MAIN {
    no_check let $pid := getpid();
    let $huge: i32 = 2_000_000_000;
    let $boom: i32 = $pid times $huge;
    no_check my.exit($boom);
}
