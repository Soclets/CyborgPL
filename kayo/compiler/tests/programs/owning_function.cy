# A function carrying what it captured with `copy` or `give` is owned like a
# list (owner, 2026-09-24: a): `copy` copies what it carries, `give` hands
# it on, and dropping it gives back what it holds. One carrying nothing, or
# only lends, copies by itself. Lent to a `lent` parameter, it stays put;
# given or copied to a `fn[…]` parameter, the callee owns it and gives it back.
%extern.C proc exit [let $code: i32];
%extern.C proc getenv [let $name: ptr[u8]] -> ptr[u8];
fn look [lent $f: fn[] -> u64] -> u64:
    ret $f();
;fn look
fn run [let $f: fn[] -> u64] -> u64:
    ret $f();
;fn run
MAIN:
    let $k := "SYNAPSE_SURELY_UNSET_VARIABLE";
    no_check let $there := ptr.is_null(getenv(lend $k));
    var $xs = [1, 2, 3];
    if $there:
        std::ls.append(lend.mut $xs, value: 4);
    ;if
    let $count := fn [] -> u64 with [copy $xs]:
        ret std::ls.len(lend $xs);
    ;fn;
    std::ls.append(lend.mut $xs, value: 5);
    let $again := copy $count;
    let $moved := give $count;
    let $words := ["a", "bb"];
    let $size := fn [] -> u64 with [give $words]:
        ret std::str.bytes(lend $words[pos: 2]);
    ;fn;
    let $inc := fn [let $a: i32] -> i32:
        ret $a + 1;
    ;fn;
    let $inc2 := $inc;
    let $inc3 := fn [] -> u64:
        ret 7;
    ;fn;
    var $got: i32 = 0;
    if $again() == 4 and $moved() == 4:
        change $got to $got + 1;
    ;if
    if std::ls.len(lend $xs) == 5 and $size() == 2:
        change $got to $got + 2;
    ;if
    if $inc2[pos: 41] == 42 and look(lend $again) == 4:
        change $got to $got + 4;
    ;if
    if run(copy $again) == 4 and run(give $again) == 4 and run($inc3) == 7:
        change $got to $got + 8;
    ;if
    no_check my.exit($got);
;MAIN
