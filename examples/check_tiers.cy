# `check`: proving a narrowing safe, with a guard the prover can see.
procedure draw [$y: float.binary.16_bit]:
    std::print.shortest("drawing ", $y, newline);
end of procedure draw
!MAIN:
    # A guardrail the compiler can see: past the `if`, the value is inside f16's range,
    # so the narrowing below is proven and `check` says so. A name never changes numeric
    # family (`types.md`), so the value is a bin from the start — `to.f16` would be the
    # way to cross from an integer, and it would fail rather than round silently.
    let $x: float.binary.32_bit = 70000.0;
    if $x greater_than 65_504.0:
        std::print.stored("too big", newline);
    end of if
    else:
        check let $y: float.binary.16_bit = $x;
        draw($y);
    end of else
    variable $xs := [1, 2, 3];
    check:
        for $i [from: 1, to: std::list.length(lend $xs), by: 1]:
            std::print.shortest($xs[position: $i], " ");
        end of for
    end of check
    std::print.stored(newline);
    !no_check change $xs[position: 2] = 9;
    # crossing families is spelled, and it fails rather than rounds (`types.md`)
    check if $x greater_than 0.0:
        change $xs[position: 1] = to.integer.signed.32_bit($x divide.nearest 2.0) otherwise 0;
    end of if
    std::print.shortest($xs, newline);
!end of MAIN
