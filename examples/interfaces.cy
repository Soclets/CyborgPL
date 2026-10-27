# Interfaces (design/interfaces.md): a named list of promises about some <T>, announced after the name.
interface Ordered <T>:
    op lt [$a: <T>, $b: <T>] -> bool;
    op eq [$a: <T>, $b: <T>] -> bool;
end of interface Ordered

interface Printable <T>:
    fn show [lent $x: <T>] -> str.utf8;
end of interface Printable

rec Vec:
    var x: float.bin.64;
    var y: float.bin.64;
end of rec Vec

# a struct keeps a promise by having the operations — the language's own operators
# may be defined for a user type (never for the language's types)
op lt [$a: Vec, $b: Vec] -> bool:
    ret ($a.x times $a.x plus $a.y times $a.y) lt ($b.x times $b.x plus $b.y times $b.y);
end of op lt
op eq [$a: Vec, $b: Vec] -> bool:
    ret $a.x eq $b.x and $a.y eq $b.y;
end of op eq
fn show [lent $v: Vec] -> str.utf8:
    ret "(" concat to.stored.str.utf8($v.x) concat ", " concat to.stored.str.utf8($v.y) concat ")";
end of fn show

Vec satisfies Ordered;        # optional: a claim the compiler checks
Vec satisfies Printable;

# a generic says what its blank accepts where it announces it (owner, 2026-10-02)
fn max <T: Ordered> [let $a: <T>, let $b: <T>] -> <T>:
    if $b lt $a:
        ret give $a;
    end of if     # a <T> may be owned, so generic code writes the verbs;
    ret give $b;                       # for a plain T (i32) the verb does nothing
end of fn max

fn describe <T: Printable> [lent $x: <T>] -> str.utf8:
    ret "value " concat show(lend $x);
end of fn describe

!MAIN:
    let $u := Vec[x = 1, y = 2];
    let $v := Vec[x = 3, y = 4];
    let $m := max(copy $u, copy $v);             # Vec is Ordered: it has < and equal_to 
    std::print.shortest(describe(lend $m), newln);
    std::print.shortest(max(3, 9), " ", max(2.5, 1.5), newln);   # numbers are Ordered by the language
    std::print.shortest($u eq $v, " ", $u lt $v, newln);     # the operators work directly too
!end of MAIN
