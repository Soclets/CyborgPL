# Collections (owner, 2026-10-13): `coll.fn` writes a function and its dot narrowings in one
# block (the other kind, `coll.handler`, is in collection_handlers.cy). The own function carries
# the collection's name (no `fn`), a member with its own body is reached as `greet.loud`, and a preset,
# a member that only fills some of the own function's arguments, is one line: `greet.hi($who)` is
# `greet(lend $who, "hi", 1)`. Private here, so no `pub` and no `bare.` mark; a `pub coll.fn`
# makes every member public under its one `bare.` mark.
coll.fn greet:
    member greet [lent $who: str.utf8, let $word: str.utf8, let $times: int.sign.32[min: 1, max: 5]] -> str.utf8 or fail[too_long]:
        if std::str.bytes(lend $who) greater_than 20:
            fail too_long;
        end of if
        var $out := str.empty;
        for $i [from: 1, to: $times, by: 1]:
            change $out to $out concat $word concat ", ";
        end of for
        ret $out concat $who;
    end of member greet

    member greet.hi [$word = "hi", $times = 1];
    member greet.cheer [$word = "hooray", $times = 3];
    member greet.loud [lent $who: str.utf8] -> str.utf8:
        ret std::str.upper.full(lend $who);
    end of member greet.loud
end of coll.fn greet

# A template (owner, 2026-10-13) makes members whose names follow a pattern. It loops with the usual
# `for` forms over values known before building, its first line `named` works out each member's name
# like text, and `kayo generate` (here `oracle --generate`) writes what it makes into this file,
# under the `#generated_N_lines_below` marker. The build only reads that part, and refuses it when it
# no longer matches the template; it is never edited by hand. Members that only fix an argument are
# presets, as `scale.times_2` here (Good practice 14): a template is for what a preset cannot make,
# such as a member whose signature changes with the value, as `zero`'s below.
const $big := 100;

coll.fn scale:
    member scale [let $x: int.sign.32, let $by: int.sign.32] -> int.sign.32:
        ret $x times $by;
    end of member scale
    member scale.times_2 [$by = 2];
    member scale.times_10 [$by = 10];
    member scale.times_100 [$by = $big];
end of coll.fn scale

# A template over types: each member returns its own type, which no preset can do. A type's name is
# text in `named`; its dots become `_`, a member's name being one word.
fn name_word [let $t: str.utf8] -> str.utf8:
    ret std::str.replace(lend $t, from: ".", to: "_");
end of fn name_word

coll.fn zero:
    for $T in [int.sign.16, int.sign.64]:
        named "of_" concat name_word($T);
        member [] -> $T:
            ret 0;
        end of member
    end of for
    #generated_6_lines_below
    member zero.of_int_sign_16 [] -> int.sign.16:
        ret 0;
    end of member zero.of_int_sign_16
    member zero.of_int_sign_64 [] -> int.sign.64:
        ret 0;
    end of member zero.of_int_sign_64
end of coll.fn zero

# `named` may call a function, run while generating, when a name needs logic (owner, 2026-10-13:
# `named size_name($n);`). A type is its name's text there.
fn width_name [let $t: str.utf8] -> str.utf8:
    if $t eq "int.sign.16":
        ret "small";
    end of if
    ret "big";
end of fn width_name

coll.fn twice:
    for $T in [int.sign.16, int.sign.64]:
        named width_name($T);
        member [let $x: $T] -> $T:
            ret $x times 2;
        end of member
    end of for
    #generated_6_lines_below
    member twice.small [let $x: int.sign.16] -> int.sign.16:
        ret $x times 2;
    end of member twice.small
    member twice.big [let $x: int.sign.64] -> int.sign.64:
        ret $x times 2;
    end of member twice.big
end of coll.fn twice

# A member whose body is written elsewhere (owner, 2026-10-19): `area.square = square_area;` is that function under
# the member's name, its path read as any path from here (`shapes.square.area` in a kit's root file).
fn square_area [let $s: int.sign.32] -> int.sign.32:
    ret $s times $s;
end of fn square_area

coll.fn area:
    member area [let $w: int.sign.32, let $h: int.sign.32] -> int.sign.32:
        ret $w times $h;
    end of member area
    member area.square = square_area;
end of coll.fn area

!MAIN:
    let $name := "Ann";
    let $a := greet(lend $name, "hello", 2) otherwise "?";
    let $b := greet.hi(lend $name) otherwise "?";
    let $c := greet.cheer(lend $name) otherwise "?";
    std::print.shortest($a, newln, $b, newln, $c, newln, greet.loud(lend $name), newln);
    let $long := "a name much longer than twenty bytes";
    let $d := greet.hi(lend $long) otherwise "too long";
    std::print.shortest($d, newln);
    std::print.shortest(scale(7, 3), " ", scale.times_2(7), " ", scale.times_10(7), " ", scale.times_100(7), " ", zero.of_int_sign_64(), " ", twice.small(300), " ", twice.big(5_000_000_000), newln);
    std::print.shortest(area(3, 4), " ", area.square(5), newln);
!end of MAIN
