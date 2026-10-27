# `check`: proving a narrowing safe, with a guard the prover can see.
proc draw [let $y: float.bin.16]:
    std::print.shortest("drawing ", $y, newln);
end of proc draw
!MAIN:
    # A guardrail the compiler can see: past the `if`, the value is inside f16's range,
    # so the narrowing below is proven and `check` says so. A name never changes numeric
    # family (`types.md`), so the value is a bin from the start — `to.f16` would be the
    # way to cross from an integer, and it would fail rather than round silently.
    let $x: float.bin.32 := 70000.0;
    if $x gt 65_504.0:
        std::print.stored("too big", newln);
    else:
        check let $y: float.bin.16 := $x;
        draw($y);
    end of if
    var $xs = [1, 2, 3];
    check:
        for $i [from: 1, to: std::ls.len(lend $xs), by: 1]:
            std::print.shortest($xs[pos: $i], " ");
        end of for
    end of check
    std::print.stored(newln);
    !no_check change $xs[pos: 2] to 9;
    # crossing families is spelled, and it fails rather than rounds (`types.md`)
    check if $x gt 0.0:
        change $xs[pos: 1] to to.int.sign.32($x div.near 2.0) otherwise 0;
    end of if
    std::print.shortest($xs, newln);
!end of MAIN
