# a module's bound is reached from outside it only when `pub`, as its other items are (owner, 2026-10-26)
module till:
    collection.bounded coin:
        coin = integer.signed.32_bit[minimum: 1, maximum: 200];
    end of collection.bounded coin
    public function show [$c: coin] -> integer.signed.32_bit:
        return $c;
    end of function show
end of module till
!MAIN:
    let $c: till.coin = 50;
    std::print.shortest(till.show($c), newline);
    let $bad: till.coin = 500;
!end of MAIN
