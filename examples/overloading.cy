# One name, several versions, picked by how many arguments and their types (owner, 2026-10-20),
# written together in a `coll.fn` (owner: b), in a `versions` block that names them once (owner, 2026-10-21).
coll.fn describe:
    versions describe:
        [let $n: int.sign.64] -> str.utf8:
            ret "a whole number";
        end of version
        [let $x: float.bin.64] -> str.utf8:
            ret "a binary float";
        end of version
        [lent $s: str.utf8] -> str.utf8:
            ret "a text";
        end of version
    end of versions describe
end of coll.fn describe

coll.fn width:
    versions width:
        [let $n: int.sign.16] -> str.utf8:
            ret "16 bits";
        end of version
        [let $n: int.sign.64] -> str.utf8:
            ret "64 bits";
        end of version
        [let $x: float.bin.32] -> str.utf8:
            ret "a f32";
        end of version
        [let $x: float.bin.64] -> str.utf8:
            ret "a f64";
        end of version
    end of versions width
end of coll.fn width

coll.fn which:
    versions which:
        [let $x: float.bin.32] -> str.utf8:
            ret "f32";
        end of version
        [let $x: float.bin.64] -> str.utf8:
            ret "f64";
        end of version
    end of versions which
end of coll.fn which

coll.fn area:
    versions area.rect:
        [let $side: int.sign.64] -> int.sign.64:
            ret $side times $side;
        end of version
        [let $w: int.sign.64, let $h: int.sign.64] -> int.sign.64:
            ret $w times $h;
        end of version
    end of versions area
end of coll.fn area

!MAIN:
    let $n: int.sign.64 := 7;
    let $x: float.bin.64 := 2.5;
    let $name := "Ann";
    std::print.shortest(describe($n), newln);
    std::print.shortest(describe($x), newln);
    std::print.shortest(describe(lend $name), newln);
    std::print.shortest(area.rect(3), " ", area.rect(3, 4), newln);
    let $small: int.sign.16 := 5;
    let $half: float.bin.32 := 0.5;
    let $y: float.bin.64 := 2.5;
    std::print.shortest(which(0.0 minus $y), " ", which(neg $half), " ", which(($half times 2.0)), newln);
    std::print.shortest(width($small), ", ", width($n), ", ", width($half), ", ", width($x), ", ", width(2.5: float.bin.32), newln);
!end of MAIN
