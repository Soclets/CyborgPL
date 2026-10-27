# The digits of pi, by the Rabinowitz–Wagon spigot: integers only, no bin anywhere.
proc pi_digits [let $n: int.sign.32]:
    let $len := (10 times $n) div.floor 3 plus 1;
    var $a := std::ls.empty;
    for $i [from: 1, to: $len, by: 1]:
        std::ls.append(lend.mut $a, value: 2);
    end of for
    var $nines := 0;
    var $predigit := 0;
    for $j [from: 1, to: $n, by: 1]:
        var $q := 0;
        for $i [from: $len, to: 1, by: neg 1]:
            let $x := (10 times $a[pos: $i]) plus ($q times $i);
            let $den := (2 times $i) minus 1;
            change $a[pos: $i] = $x mod.floor $den;
            change $q = $x div.floor $den;
        end of for
        change $a[pos: 1] = $q mod.floor 10;
        change $q = $q div.floor 10;
        if $q eq 9:
            change $nines = $nines plus 1;
        end of if
        else if $q eq 10:
            std::print.shortest($predigit plus 1);
            for $k [from: 1, to: $nines, by: 1]:
                std::print.shortest(0);
            end of for
            change $predigit = 0;
            change $nines = 0;
        end of else if
        else:
            if $j greater_than 1:
                std::print.shortest($predigit);
            end of if
            if $j eq 2:
                std::print.shortest(".");
            end of if
            change $predigit = $q;
            for $k [from: 1, to: $nines, by: 1]:
                std::print.shortest(9);
            end of for
            change $nines = 0;
        end of else
    end of for
    std::print.shortest($predigit, newln);
end of proc pi_digits

!MAIN:
    pi_digits(60);
!end of MAIN
