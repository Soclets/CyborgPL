# Modules as blocks in one file (a kit of one root file). See examples/geometry/ for files.


const $scale := 10;

module counter:
    pub const $start := 1;
    pub rec Count:
        var n: i32;
    ;rec Count
    pub fn bump [let $c: Count] -> Count:
        ret Count[n = $c.n + step()];
    ;fn bump
    fn step [] -> i32:
        ret my.$scale;
    ;fn step      # private; reaches the root's const
    pub.kit module inner:
        pub.kit fn twice [let $n: i32] -> i32:
            ret $n x 2;
        ;fn twice
    ;module inner
;module counter

proc show [let $c: counter.Count]:
    std::print("count ", $c.n, newln);
;proc show

MAIN:
    let $c := counter.Count[n = counter.$start];
    show(copy $c);
    let $d := counter.bump(give $c);
    show(copy $d);
    std::print(counter.inner.twice($d.n), newln);
;MAIN
