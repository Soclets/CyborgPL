# Finding 2026-10-15 (check run, seed 1088, cut down by hand): a number into brain16 rounds twice in
# the oracle. 25262379159781017 lies between 179 x 2^47 and 180 x 2^47, 70368744177305 above the
# lower, where half a step is 70368744177664: the nearest brain16 is 179 x 2^47 =
# 25192010415603712 (types.md: a literal "rounds to the nearest brain16", to.brain16 rounds).
# Zomp prints that. The oracle prints 180 x 2^47 = 25332747903959040: rounded to bin32 first
# (exactly half way, 25262379159781376), then ties to even. The same for the literal and for
# to.brain16 from an int64; 'w' times 0.1 then differs too (2515682604351488 against
# 2533274790395904, the line the seed's run caught).
MAIN {
    let 'v': int64 = 25262379159781017;
    let 'w' := to.brain16['v'];
    std::print.stored['w', newln];
    let 'l': brain16 = 25262379159781017.0;
    std::print.stored['l', newln];
    let 'x': brain16 = 'w' times 0.1;
    std::print.stored['x', newln];
}
