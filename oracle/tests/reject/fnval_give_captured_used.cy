MAIN {
    let 's' := "text";
    let 'f' := fn [] -> i32 with [give 's'] { return std::str.bytes[lend 's']; };
    std::print.stored['s'];
}
