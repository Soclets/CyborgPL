fn sum [let ...$nums: i32] -> i32:
    ret 0;
;fn sum
MAIN:
    var $zs = [1, 2];
    std::ls.append(lend.mut $zs, value: 3);      # no bound: length no longer known here
    let $t := sum(...$zs);
;MAIN
