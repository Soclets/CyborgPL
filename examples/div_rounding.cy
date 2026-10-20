# Every division says how it rounds (owner, 2026-10-02): `floor` down, `ceil` up, `trunc` toward
# zero, `near` to the nearest (a tie to even). `mod.X` is what `div.X` leaves over:
# a equal_to (a div.X b) times b plus (a mod.X b). On numbers with a fraction, `div.near` is ordinary division
# and `floor`/`ceil`/`trunc` give a whole quotient.
MAIN {
    std::print.useful[(7 div.floor 2), " ", (7 div.ceil 2), " ", (7 div.trunc 2), " ", (7 div.near 2), newln];
    std::print.useful[(-7 div.floor 2), " ", (-7 div.ceil 2), " ", (-7 div.trunc 2), " ", (-7 div.near 2), newln];
    std::print.useful[(7 mod.floor 2), " ", (7 mod.ceil 2), " ", (7 mod.trunc 2), " ", (7 mod.near 2), newln];
    std::print.useful[(-7 mod.floor 2), " ", (-7 mod.ceil 2), " ", (-7 mod.trunc 2), " ", (-7 mod.near 2), newln];
    std::print.useful[(5 div.near 2), " ", (5 mod.near 2), newln];        # 2.5 is a tie: to the even 2, 1 left
    let 'items': i32 = 7;
    let 'per_page': i32 = 2;
    std::print.useful["pages: ", ('items' div.ceil 'per_page'), newln];
    let 'board': f64 = 7.5;
    let 'plank': f64 = 2.0;
    std::print.useful[('board' div.near 'plank'), " ", ('board' div.floor 'plank'), " ", ('board' mod.floor 'plank'), " ", ('board' div.ceil 'plank'), " ", ('board' mod.ceil 'plank'), newln];
    let 'p': d64 = -7.5;
    let 'q': d64 = 2.0;
    std::print.useful[('p' div.near 'q'), " ", ('p' div.floor 'q'), " ", ('p' mod.floor 'q'), " ", ('p' div.trunc 'q'), " ", ('p' mod.trunc 'q'), " ", ('p' div.near 'q'), newln];
}
