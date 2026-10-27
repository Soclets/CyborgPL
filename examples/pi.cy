# The digits of pi, by the Rabinowitz–Wagon spigot: integers only, no bin anywhere.
procedure pi_digits [$n: integer.signed.32_bit]:
    let $len := (10 times $n) divide.floor 3 plus 1;
    variable $a := std::list.empty;
    for $i [from: 1, to: $len, by: 1]:
        std::list.append(lend.mutable $a, value: 2);
    end of for
    variable $nines := 0;
    variable $predigit := 0;
    for $j [from: 1, to: $n, by: 1]:
        variable $q := 0;
        for $i [from: $len, to: 1, by: negative 1]:
            let $x := (10 times $a[position: $i]) plus ($q times $i);
            let $den := (2 times $i) minus 1;
            change $a[position: $i] = $x modulo.floor $den;
            change $q = $x divide.floor $den;
        end of for
        change $a[position: 1] = $q modulo.floor 10;
        change $q = $q divide.floor 10;
        if $q equal_to 9:
            change $nines = $nines plus 1;
        end of if
        else if $q equal_to 10:
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
            if $j equal_to 2:
                std::print.shortest(".");
            end of if
            change $predigit = $q;
            for $k [from: 1, to: $nines, by: 1]:
                std::print.shortest(9);
            end of for
            change $nines = 0;
        end of else
    end of for
    std::print.shortest($predigit, newline);
end of procedure pi_digits

!MAIN:
    pi_digits(60);
!end of MAIN
