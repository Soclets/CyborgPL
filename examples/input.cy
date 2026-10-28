# Input and range types (design/input.md). Try: echo 42 | oracle input.cy
MAIN:
    let $text := std::input.line() ?? "(no input)";     # a boundary: fails with `closed`
    std::print.stored("got: ", $text, newln);

    # into a range type, the conversion fails on a bad number or an out-of-range one —
    # a failure to handle, never a crash; past it 'n' is provably 0..100
    let $n := to.i32[min: 0, max: 100]($text) ?? 50;
    std::print("n = ", $n, newln);
;MAIN
