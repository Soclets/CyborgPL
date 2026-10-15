MAIN {
    let immut 'buf': str.utf8 = "hi";
    no_check inline.ASM {
        strb wzr, [x1]
    } with [lend.mut 'buf' -> x1];
}
