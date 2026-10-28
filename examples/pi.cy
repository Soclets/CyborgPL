# The digits of pi, by the Rabinowitz–Wagon spigot: integers only, no bin anywhere.
proc pi_digits [let $n: i32]:
    let $len := (10 x $n) div.floor 3 + 1;
    var $a = std::ls.empty;
    for $i [from: 1, to: $len, by: 1]:
        std::ls.append(lend.mut $a, value: 2);
    ;for
    var $nines = 0;
    var $predigit = 0;
    for $j [from: 1, to: $n, by: 1]:
        var $q = 0;
        for $i [from: $len, to: 1, by: -1]:
            let $x := (10 x $a[pos: $i]) + ($q x $i);
            let $den := (2 x $i) - 1;
            change $a[pos: $i] to $x mod.floor $den;
            change $q to $x div.floor $den;
        ;for
        change $a[pos: 1] to $q mod.floor 10;
        change $q to $q div.floor 10;
        if $q == 9:
            change $nines to $nines + 1;
        else if $q == 10:
            std::print($predigit + 1);
            for $k [from: 1, to: $nines, by: 1]:
                std::print(0);
            ;for
            change $predigit to 0;
            change $nines to 0;
        else:
            if $j > 1:
                std::print($predigit);
            ;if
            if $j == 2:
                std::print(".");
            ;if
            change $predigit to $q;
            for $k [from: 1, to: $nines, by: 1]:
                std::print(9);
            ;for
            change $nines to 0;
        ;if
    ;for
    std::print($predigit, newln);
;proc pi_digits

MAIN:
    pi_digits(60);
;MAIN
