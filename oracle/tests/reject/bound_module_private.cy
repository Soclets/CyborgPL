# a module's bound is reached from outside it only when `pub`, as its other items are (owner, 2026-10-26)
module till {
    coll.bound coin {
        coin = i32[min: 1, max: 200];
    }
    pub fn show [$c: coin] -> i32 { return $c; }
}
MAIN {
    let $c: till.coin = 50;
    std::print.useful(till.show($c), newln);
    let $bad: till.coin = 500;
}
