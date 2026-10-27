# Modules as blocks in one file (a kit of one root file). See examples/geometry/ for files.


constant $scale := 10;

module counter:
    public constant $start := 1;
    public record Count:
        n: integer.signed.32_bit;
    end of record Count
    public function bump [$c: Count] -> Count:
        return Count[n = $c.n plus step()];
    end of function bump
    function step [] -> integer.signed.32_bit:
        return my.$scale;
    end of function step      # private; reaches the root's const
    public.kit module inner:
        public.kit function twice [$n: integer.signed.32_bit] -> integer.signed.32_bit:
            return $n times 2;
        end of function twice
    end of module inner
end of module counter

procedure show [$c: counter.Count]:
    std::print.shortest("count ", $c.n, newline);
end of procedure show

!MAIN:
    let $c := counter.Count[n = counter.$start];
    show(copy $c);
    let $d := counter.bump(give $c);
    show(copy $d);
    std::print.shortest(counter.inner.twice($d.n), newline);
!end of MAIN
