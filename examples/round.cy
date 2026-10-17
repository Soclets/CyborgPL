# Rounding is one family, `decimals` optional (owner, 2026-10-17: "(c)"); a bare `round` is refused, since it
# names no way to round a tie (owner: "(a)"). A bin is rounded from its exact digits: 2.675 is a little under.
MAIN {
    let 'y': bin64 = 2.5;
    std::print.useful[round.near['y'], " ", round.half_up['y'], " ", round.floor['y'], " ", round.ceil['y'], " ", round.trunc[0.0 - 'y'], newln];
    std::print.useful[round.near[2.675, decimals 2], " ", round.floor[-1.234, decimals 1], newln];
    let 'd' := exactly 2.675;
    std::print.useful[round.near['d', decimals 2], " ", round.half_up['d', decimals 2], " ", round.floor[0.0 - 'd', decimals 2], " ", round.ceil['d', decimals 1], newln];
    std::print.useful[round.near[exactly 2.665, decimals 2], " ", round.trunc[exactly 2.665, decimals 1], newln];
}
