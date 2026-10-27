# One name, several versions, picked by how many arguments and their types (owner, 2026-10-20),
# written together in a `coll.fn` (owner: b), in a `versions` block that names them once (owner, 2026-10-21).
collection.function describe:
    versions describe:
        [$n: integer.signed.64_bit] -> string.utf8:
            return "a whole number";
        end of version
        [$x: float.binary.64_bit] -> string.utf8:
            return "a binary float";
        end of version
        [lent $s: string.utf8] -> string.utf8:
            return "a text";
        end of version
    end of versions describe
end of collection.function describe

collection.function width:
    versions width:
        [$n: integer.signed.16_bit] -> string.utf8:
            return "16 bits";
        end of version
        [$n: integer.signed.64_bit] -> string.utf8:
            return "64 bits";
        end of version
        [$x: float.binary.32_bit] -> string.utf8:
            return "a f32";
        end of version
        [$x: float.binary.64_bit] -> string.utf8:
            return "a f64";
        end of version
    end of versions width
end of collection.function width

collection.function which:
    versions which:
        [$x: float.binary.32_bit] -> string.utf8:
            return "f32";
        end of version
        [$x: float.binary.64_bit] -> string.utf8:
            return "f64";
        end of version
    end of versions which
end of collection.function which

collection.function area:
    versions area.rect:
        [$side: integer.signed.64_bit] -> integer.signed.64_bit:
            return $side times $side;
        end of version
        [$w: integer.signed.64_bit, $h: integer.signed.64_bit] -> integer.signed.64_bit:
            return $w times $h;
        end of version
    end of versions area
end of collection.function area

!MAIN:
    let $n: integer.signed.64_bit = 7;
    let $x: float.binary.64_bit = 2.5;
    let $name := "Ann";
    std::print.shortest(describe($n), newline);
    std::print.shortest(describe($x), newline);
    std::print.shortest(describe(lend $name), newline);
    std::print.shortest(area.rect(3), " ", area.rect(3, 4), newline);
    let $small: integer.signed.16_bit = 5;
    let $half: float.binary.32_bit = 0.5;
    let $y: float.binary.64_bit = 2.5;
    std::print.shortest(which(0.0 minus $y), " ", which(negate $half), " ", which(($half times 2.0)), newline);
    std::print.shortest(width($small), ", ", width($n), ", ", width($half), ", ", width($x), ", ", width(2.5: float.binary.32_bit), newline);
!end of MAIN
