# Symbols (owner, 2026-10-14): each spells its word's most useful general form. Good practice writes the
# words; the symbols are for whoever thinks in them.
#   + - *        plus, minus, times          / %     div.trunc, mod.trunc (toward zero)
#   == != <= >=  equal_to, not_equal, at_most, at_least      **   pow
#   & && ||      and, and.then, or.else between two values
#   &'x', &mut 'x'   lend / lent, lend.mut / lent.mut
#   *const T, *mut T, *'p', change *'p' = v;   ptr[T], ptr.mut[T], ptr.read, ptr.write
#   exactly 19.99, *19.99, exactly 1000        an exact deci
extern.C fn malloc ['bytes': uint64] -> *mut uint8;
extern.C fn free ['p': *mut uint8];

fn count [&'s': str.utf8] -> uint64 { return str.bytes[&'s']; }
fn grow [&mut 'xs': list[int32]] { list.append[&mut 'xs', value 1]; }

MAIN {
    let 'a' := 7;
    let 'b' := 2;
    std::print.useful['a' + 'b', " ", 'a' - 'b', " ", 'a' * 'b', " ", 'a' / 'b', " ", 'a' % 'b', newln];
    std::print.useful[-7 / 2, " ", -7 % 2, " ", 'b' ** 10, " ", 1 + 2 * 3, newln];
    std::print.useful['a' == 'b', " ", 'a' != 'b', " ", 'a' <= 'b', " ", 'a' >= 'b', newln];
    std::print.useful[('a' > 1) & ('b' > 1), " ", ('a' > 9) && ('b' > 1), " ", ('a' > 9) || ('b' > 1), newln];
    let 'xs': list[int32] = [5,];
    grow[&mut 'xs'];
    let 'h' := "hello";
    std::print.useful[list.length[&'xs'], " ", count[&'h'], newln];
    std::print.useful[exactly 19.99, " ", exactly 1000, " ", *0.1, newln];
    no_check let 'p' := malloc[4];
    no_check change *'p' = 65;
    no_check let 'c' := *'p';
    std::print.useful['c', newln];
    no_check free['p'];
}
