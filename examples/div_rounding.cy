# Every division says how it rounds (owner, 2026-10-02): `floor` down, `ceil` up, `trunc` toward
# zero, `near` to the nearest (a tie to even). `mod.X` is what `div.X` leaves over:
# a equal_to (a div.X b) times b plus (a mod.X b). On numbers with a fraction, `div.near` is ordinary division
# and `floor`/`ceil`/`trunc` give a whole quotient.
!MAIN:
    std::print.shortest((7 div.floor 2), " ", (7 div.ceil 2), " ", (7 div.trunc 2), " ", (7 div.near 2), newln);
    std::print.shortest((neg 7 div.floor 2), " ", (neg 7 div.ceil 2), " ", (neg 7 div.trunc 2), " ", (neg 7 div.near 2), newln);
    std::print.shortest((7 mod.floor 2), " ", (7 mod.ceil 2), " ", (7 mod.trunc 2), " ", (7 mod.near 2), newln);
    std::print.shortest((neg 7 mod.floor 2), " ", (neg 7 mod.ceil 2), " ", (neg 7 mod.trunc 2), " ", (neg 7 mod.near 2), newln);
    std::print.shortest((5 div.near 2), " ", (5 mod.near 2), newln);        # 2.5 is a tie: to the even 2, 1 left
    let $items: int.sign.32 := 7;
    let $per_page: int.sign.32 := 2;
    std::print.shortest("pages: ", ($items div.ceil $per_page), newln);
    let $board: float.bin.64 := 7.5;
    let $plank: float.bin.64 := 2.0;
    std::print.shortest(($board div.near $plank), " ", ($board div.floor $plank), " ", ($board mod.floor $plank), " ", ($board div.ceil $plank), " ", ($board mod.ceil $plank), newln);
    let $p: float.%deci.64 := neg 7.5;
    let $q: float.%deci.64 := 2.0;
    std::print.shortest(($p div.near $q), " ", ($p div.floor $q), " ", ($p mod.floor $q), " ", ($p div.trunc $q), " ", ($p mod.trunc $q), " ", ($p div.near $q), newln);
!end of MAIN
