MAIN {
    let 's' := "text";
    let 'f' := fn [] -> int32 with [give 's'] { return str.bytes[lend 's']; };
    std::print.stored['s'];
}
