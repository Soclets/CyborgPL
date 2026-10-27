# Modules as blocks in one file (a kit of one root file). See examples/geometry/ for files.


const $scale := 10;

module counter:
    pub const $start := 1;
    pub rec Count:
        n: int.sign.32;
    end of rec Count
    pub fn bump [$c: Count] -> Count:
        ret Count[n = $c.n plus step()];
    end of fn bump
    fn step [] -> int.sign.32:
        ret my.$scale;
    end of fn step      # private; reaches the root's const
    pub.kit module inner:
        pub.kit fn twice [$n: int.sign.32] -> int.sign.32:
            ret $n times 2;
        end of fn twice
    end of module inner
end of module counter

proc show [$c: counter.Count]:
    std::print.shortest("count ", $c.n, newln);
end of proc show

!MAIN:
    let $c := counter.Count[n = counter.$start];
    show(copy $c);
    let $d := counter.bump(give $c);
    show(copy $d);
    std::print.shortest(counter.inner.twice($d.n), newln);
!end of MAIN
