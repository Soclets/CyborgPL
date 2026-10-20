# Lists the Nerve cord (owner, 2026-10-14): `std::twice` and bare `twice` are its, `std::print` falls
# back to the built-in, which it has not written, its module `str` is reached as `std::str.shout` and `std::str.at.byte[…, pos 2]` (2 + 103), and its `exit` takes over, so the run ends 0, not 7.

MAIN {
    let 'xs': list[i64] = [1, 2, 3];
    std::print.useful[std::twice[4], " ", twice[5], " ", helper::doubled[6], " ", total[lend 'xs'], newln];
    let 'h' := "hi";
    let 'i' := std::str.at.byte[lend 'h', pos: 2] otherwise 0;
    std::print.useful[std::str.shout[lend 'h'], " ", 'i', newln];
    std::exit[7];
}
