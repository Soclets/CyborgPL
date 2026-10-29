# An average of i32 values fits an i32 (precog.md: "a sum bounded by n x
# max divided by n is at most max"), for a list whose length and contents
# come from outside. The final build accepts it with no checks left.
%extern.C proc exit [let $code: i32];
%extern.C proc getenv [let $name: ptr[u8]] -> ptr[u8];
%extern.C proc atoi [let $s: ptr[u8]] -> i32;
fn average [lent $xs: std::ls[i32]] -> i32 or fail[empty]:
    if std::ls.len(lend $xs) == 0:
        fail empty;
    ;if
    var $total = 0;
    for $x in lend $xs:
        change $total to $total + $x;
    ;for
    ret $total div.floor std::ls.len(lend $xs);
;fn average
MAIN:
    let $k := "N";
    no_check let $n: i32 := atoi(getenv(lend $k));
    var $xs: std::ls[i32][min: 0, max: 1000] = std::ls.empty;
    var $i: i32 = 0;
    while $i < $n and $i < 1000:
        std::ls.append(lend.mut $xs, value: ($n mod.floor 1000) - $i);
        change $i to $i + 1;
    ;while
    let $a := average(lend $xs) ?? 0;
    no_check my.exit($a mod.floor 256);
;MAIN
