# Interfaces (design/interfaces.md): a named list of promises about some <T>, announced after the name.
interface Ordered <T>:
    operator less_than [$a: <T>, $b: <T>] -> boolean;
    operator equal_to [$a: <T>, $b: <T>] -> boolean;
end of interface Ordered

interface Printable <T>:
    function show [lent $x: <T>] -> string.utf8;
end of interface Printable

record Vec:
    x: float.binary.64_bit;
    y: float.binary.64_bit;
end of record Vec

# a struct keeps a promise by having the operations — the language's own operators
# may be defined for a user type (never for the language's types)
operator less_than [$a: Vec, $b: Vec] -> boolean:
    return ($a.x times $a.x plus $a.y times $a.y) less_than ($b.x times $b.x plus $b.y times $b.y);
end of operator <
operator equal_to [$a: Vec, $b: Vec] -> boolean:
    return $a.x equal_to $b.x and $a.y equal_to $b.y;
end of operator equal_to
function show [lent $v: Vec] -> string.utf8:
    return "(" concatenate to.stored.string.utf8($v.x) concatenate ", " concatenate to.stored.string.utf8($v.y) concatenate ")";
end of function show

Vec satisfies Ordered;        # optional: a claim the compiler checks
Vec satisfies Printable;

# a generic says what its blank accepts where it announces it (owner, 2026-10-02)
function max <T: Ordered> [$a: <T>, $b: <T>] -> <T>:
    if $b less_than $a:
        return give $a;
    end of if     # a <T> may be owned, so generic code writes the verbs;
    return give $b;                       # for a plain T (i32) the verb does nothing
end of function max

function describe <T: Printable> [lent $x: <T>] -> string.utf8:
    return "value " concatenate show(lend $x);
end of function describe

!MAIN:
    let $u := Vec[x = 1, y = 2];
    let $v := Vec[x = 3, y = 4];
    let $m := max(copy $u, copy $v);             # Vec is Ordered: it has < and equal_to 
    std::print.shortest(describe(lend $m), newline);
    std::print.shortest(max(3, 9), " ", max(2.5, 1.5), newline);   # numbers are Ordered by the language
    std::print.shortest($u equal_to $v, " ", $u less_than $v, newline);     # the operators work directly too
!end of MAIN
