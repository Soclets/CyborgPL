# a module's bound is reached from outside it only when `pub`, as its other items are (owner, 2026-10-26)
module till:
    coll.bounded coin:
        coin = int.sign.32[min: 1, max: 200];
    end of coll.bounded coin
    pub fn show [let $c: coin] -> int.sign.32:
        ret $c;
    end of fn show
end of module till
!MAIN:
    let $c: till.coin = 50;
    std::print.shortest(till.show($c), newln);
    let $bad: till.coin = 500;
!end of MAIN
