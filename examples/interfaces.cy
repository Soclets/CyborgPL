# Interfaces (design/interfaces.md): a named list of promises about some <T>, announced after the name.
sig Ordered <T>:
    op < [lent $a: <T>, lent $b: <T>] -> bool;
    op == [lent $a: <T>, lent $b: <T>] -> bool;
;sig Ordered

sig Printable <T>:
    fn show [lent $x: <T>] -> str.utf8;
;sig Printable

rec Vec:
    var x: f64;
    var y: f64;
;rec Vec

# a struct keeps a promise by having the operations — the language's own operators
# may be defined for a user type (never for the language's types)
op < [lent $a: Vec, lent $b: Vec] -> bool:
    ret ($a.x x $a.x + $a.y x $a.y) < ($b.x x $b.x + $b.y x $b.y);
;op <
op == [lent $a: Vec, lent $b: Vec] -> bool:
    ret $a.x == $b.x & $a.y == $b.y;
;op ==
fn show [lent $v: Vec] -> str.utf8:
    ret "(" ++ to.stored.str.utf8($v.x) ++ ", " ++ to.stored.str.utf8($v.y) ++ ")";
;fn show

Vec satisfies Ordered;        # optional: a claim the compiler checks
Vec satisfies Printable;

# a generic says what its blank accepts where it announces it (owner, 2026-10-02)
fn max <T: Ordered> [let $a: <T>, let $b: <T>] -> <T>:
    if $b < $a:
        ret give $a;
    ;if     # a <T> may be owned, so generic code writes the verbs;
    ret give $b;                       # for a plain T (i32) the verb does nothing
;fn max

fn describe <T: Printable> [lent $x: <T>] -> str.utf8:
    ret "value " ++ show(lend $x);
;fn describe

MAIN:
    let $u := Vec[x = 1, y = 2];
    let $v := Vec[x = 3, y = 4];
    let $m := max(copy $u, copy $v);             # Vec is Ordered: it has < and equal_to 
    std::println(describe(lend $m));
    std::println(max(3, 9), " ", max(2.5, 1.5));   # numbers are Ordered by the language
    std::println($u == $v, " ", $u < $v);     # the operators work directly too
;MAIN
