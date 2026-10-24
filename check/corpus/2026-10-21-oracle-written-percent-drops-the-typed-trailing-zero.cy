# Finding 2026-10-21 (generator, seed 203, the print.written episode): `print.written` prints a value
# as its literal was typed (owner, 2026-10-17; design/strings.md), and `exactly` is not printed
# (oracle/README.md). Typed `exactly 51.0%`, kayo prints "51.0%"; the oracle prints "51%", dropping
# the `.0` it keeps for the f16 beside it ("0.337403770").
MAIN {
    let 'tx' := exactly 51.0%;
    let 'wx': f16 = 0.337403770;
    std::print.written('wx', " ", 'tx', " ", -2.50, newln);
}
