# Variadics (functions.md): a bundle of any number of values, typed or each
# its own; lent or owned; `...'xs'` unpacking a list (literal, bounded with
# a length known only at run time, or a bundle passed on), `...copy` and
# `...give` of owned elements. The call builds the bundle.
%extern.C proc exit [$code: i32];
%extern.C proc getenv [$name: ptr[u8]] -> ptr[u8];
fn sum [...$nums] -> f64:
    var $total: f64 = 0.0;
    for $n in lend $nums:
        change $total to $total + to.f64($n);
    ;for
    ret $total;
;fn sum
fn add [...$nums: i32] -> i32:
    var $total: i32 = 0;
    for $n in lend $nums:
        change $total to $total + $n;
    ;for
    ret $total;
;fn add
fn count [...$nums: i32] -> u64:
    ret std::ls.len(lend $nums);
;fn count
fn bytes [lent $label: str.utf8, lent ...$parts: str.utf8] -> u64:
    var $n: u64 = std::str.bytes(lend $label);
    for $p in lend $parts:
        change $n to $n + std::str.bytes(lend $p);
    ;for
    ret $n;
;fn bytes
fn pass_on [lent ...$parts: str.utf8] -> u64:
    ret bytes("x", ...$parts);
;fn pass_on
fn keep [...$names: str.utf8] -> u64:
    ret std::ls.len(lend $names);
;fn keep
MAIN:
    let $k := "SYNAPSE_SURELY_UNSET_VARIABLE";
    no_check let $there := ptr.is_null(getenv(lend $k));
    var $got: i32 = 0;
    var $three: i32 = 4;
    if $there:
        change $three to 3;
    ;if
    if sum(1, 2.5, $three) == 6.5 and sum() == 0.0:
        change $got to $got + 1;
    ;if
    if add(4, 5, $three) == 12 and count(4, 5, 6) == 3 and count() == 0:
        change $got to $got + 2;
    ;if
    let $xs := [1, 2, $three];
    if add(...$xs) == 6 and add(10, ...$xs, 20) == 36:
        change $got to $got + 4;
    ;if
    var $ys: std::ls[i32][min: 0, max: 4] = std::ls.empty;
    std::ls.append(lend.mut $ys, value: 7);
    if $there:
        std::ls.append(lend.mut $ys, value: 8);
    ;if
    if add(...$ys) == 15 and count(...$ys, 1) == 3:
        change $got to $got + 8;
    ;if
    let $name := "Ann";
    if bytes("mixed", lend $name, "yo") == 10 and bytes("none") == 4 and pass_on(lend $name, "b") == 5:
        change $got to $got + 16;
    ;if
    var $words = ["a", "bb"];
    if $there:
        change $words to ["a", "bcd"];
    ;if
    if keep(...copy $words) == 2 and std::ls.len(lend $words) == 2:
        change $got to $got + 32;
    ;if
    if keep(...give $words) == 2 and std::ls.len(lend $words) == 0:
        change $got to $got + 64;
    ;if
    no_check my.exit($got);
;MAIN
