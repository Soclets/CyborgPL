# Collections (owner, 2026-10-13): `coll.fn` writes a function and its dot narrowings in one
# block (the other kind, `coll.handler`, is in collection_handlers.cy). The own function carries
# the collection's name (no `fn`), a member with its own body is reached as `greet.loud`, and a preset,
# a member that only fills some of the own function's arguments, is one line: `greet.hi($who)` is
# `greet(lend $who, "hi", 1)`. Private here, so no `pub` and no `bare.` mark; a `pub coll.fn`
# makes every member public under its one `bare.` mark.
collection.function greet:
    greet [lent $who: string.utf8, $word: string.utf8, $times: integer.signed.32_bit[minimum: 1, maximum: 5]] -> string.utf8 or fail[too_long]:
        if std::string.bytes(lend $who) greater_than 20:
            fail too_long;
        end of if
        variable $out := string.empty;
        for $i [from: 1, to: $times, by: 1]:
            change $out = $out concatenate $word concatenate ", ";
        end of for
        return $out concatenate $who;
    end of greet

    greet.hi [$word = "hi", $times = 1];
    greet.cheer [$word = "hooray", $times = 3];
    greet.loud [lent $who: string.utf8] -> string.utf8:
        return std::string.uppercase.full(lend $who);
    end of greet.loud
end of collection.function greet

# A template (owner, 2026-10-13) makes members whose names follow a pattern. It loops with the usual
# `for` forms over values known before building, its first line `named` works out each member's name
# like text, and `kayo generate` (here `oracle --generate`) writes what it makes into this file,
# under the `#generated_N_lines_below` marker. The build only reads that part, and refuses it when it
# no longer matches the template; it is never edited by hand. Members that only fix an argument are
# presets, as `scale.times_2` here (Good practice 14): a template is for what a preset cannot make,
# such as a member whose signature changes with the value, as `zero`'s below.
constant $big := 100;

collection.function scale:
    scale [$x: integer.signed.32_bit, $by: integer.signed.32_bit] -> integer.signed.32_bit:
        return $x times $by;
    end of scale
    scale.times_2 [$by = 2];
    scale.times_10 [$by = 10];
    scale.times_100 [$by = $big];
end of collection.function scale

# A template over types: each member returns its own type, which no preset can do. A type's name is
# text in `named`; its dots become `_`, a member's name being one word.
function name_word [$t: string.utf8] -> string.utf8:
    return std::string.replace(lend $t, from: ".", to: "_");
end of function name_word

collection.function zero:
    for $T in [integer.signed.16_bit, integer.signed.64_bit]:
        named "of_" concatenate name_word($T);
        [] -> $T:
            return 0;
        end of named
    end of for
    #generated_6_lines_below
    zero.of_integer_signed_16_bit [] -> integer.signed.16_bit:
        return 0;
    end of zero.of_integer_signed_16_bit
    zero.of_integer_signed_64_bit [] -> integer.signed.64_bit:
        return 0;
    end of zero.of_integer_signed_64_bit
end of collection.function zero

# `named` may call a function, run while generating, when a name needs logic (owner, 2026-10-13:
# `named size_name($n);`). A type is its name's text there.
function width_name [$t: string.utf8] -> string.utf8:
    if $t equal_to "integer.signed.16_bit":
        return "small";
    end of if
    return "big";
end of function width_name

collection.function twice:
    for $T in [integer.signed.16_bit, integer.signed.64_bit]:
        named width_name($T);
        [$x: $T] -> $T:
            return $x times 2;
        end of named
    end of for
    #generated_6_lines_below
    twice.small [$x: integer.signed.16_bit] -> integer.signed.16_bit:
        return $x times 2;
    end of twice.small
    twice.big [$x: integer.signed.64_bit] -> integer.signed.64_bit:
        return $x times 2;
    end of twice.big
end of collection.function twice

# A member whose body is written elsewhere (owner, 2026-10-19): `area.square = square_area;` is that function under
# the member's name, its path read as any path from here (`shapes.square.area` in a kit's root file).
function square_area [$s: integer.signed.32_bit] -> integer.signed.32_bit:
    return $s times $s;
end of function square_area

collection.function area:
    area [$w: integer.signed.32_bit, $h: integer.signed.32_bit] -> integer.signed.32_bit:
        return $w times $h;
    end of area
    area.square = square_area;
end of collection.function area

!MAIN:
    let $name := "Ann";
    let $a := greet(lend $name, "hello", 2) otherwise "?";
    let $b := greet.hi(lend $name) otherwise "?";
    let $c := greet.cheer(lend $name) otherwise "?";
    std::print.shortest($a, newline, $b, newline, $c, newline, greet.loud(lend $name), newline);
    let $long := "a name much longer than twenty bytes";
    let $d := greet.hi(lend $long) otherwise "too long";
    std::print.shortest($d, newline);
    std::print.shortest(scale(7, 3), " ", scale.times_2(7), " ", scale.times_10(7), " ", scale.times_100(7), " ", zero.of_integer_signed_64_bit(), " ", twice.small(300), " ", twice.big(5_000_000_000), newline);
    std::print.shortest(area(3, 4), " ", area.square(5), newline);
!end of MAIN
