# Generics announced after the name, each saying what it accepts (owner, 2026-10-02). An interface
# may list types, one per line, as well as operations; a blank may need several, joined with `and`;
# lifetime groups come after the generics, a `|` between the two kinds.
interface Number {
    i32
    i64
    f64
}

interface Ordered <T> {
    op < ['a': <T>, 'b': <T>] -> bool;
    op equal_to ['a': <T>, 'b': <T>] -> bool;
}

interface WholeOrdered <T> {
    i32
    i64
    op < ['a': <T>, 'b': <T>] -> bool;
}

fn bigger <T: Number and Ordered> ['a': <T>, 'b': <T>] -> <T> {
    if 'b' < 'a' { return 'a'; }
    return 'b';
}

fn smaller <T: WholeOrdered> ['a': <T>, 'b': <T>] -> <T> {
    if 'a' < 'b' { return 'a'; }
    return 'b';
}

fn whole <T: any> [@x] [lent @x 'xs': list[<T>]] -> lent @x list[<T>] { return lend 'xs'; }

struct Box <T: Number> { item: <T> }

MAIN {
    std::print.useful[bigger[3, 9], " ", bigger[2.5, 1.5], " ", smaller[3, 9], newln];
    let 'xs' := ["a", "b"];
    std::print.useful[whole[lend 'xs'], newln];
    let 'b': Box[i32] = Box[item = 4];
    std::print.useful['b'.item, newln];
}
