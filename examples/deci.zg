# deci, IEEE 754 decimal (owner, 2026-09-29): 0.1 plus 0.2 is exactly 0.3; every operation rounds to the
# width half to even; trailing zeros are kept (print shows 2.50, print.useful 2.5); a literal with too
# many digits is an error; text, and an integer whose type could have too many digits, fail
# too_many_digits; to.useful.deci64[bin] is the shortest decimal, to.stored.deci64[bin] the exact value (the form is always written).
MAIN {
    let 'a': deci64 = 0.1;
    let 'b': deci64 = 'a' plus 0.2;
    std::print.stored['b', " ", 'b' equal_to 0.3, newln];
    let 'price': deci64 = 19.99;
    let 'sub': deci64 = 'price' times 3;
    let 'tax': deci64 = 'sub' times 0.075;
    std::print.stored['sub', " ", 'tax', " ", 'sub' plus 'tax', newln];
    let 'h': deci64 = 2.50;
    std::print.stored['h' times 2, " ", 'h', " ", 1.50 equal_to 1.5, newln];
    std::print.useful['h' times 2, " ", 'h', newln];
    let 'third': deci64 = 1;
    std::print.stored['third' div.near 3, " ", 'h' div.near 2, " ", 10.0 div.near 4, newln];
    let 'r': deci32 = 2.5;
    let 'e1': deci32 = 1.000005;
    std::print.stored['e1' times 1, " ", 'e1' plus 0.0000005, newln];
    let 'w': bin64 = 0.1;
    std::print.stored[to.stored.deci64['w'], " ", to.stored.deci128['w'], " ", to.useful.deci128['w'], newln];
    let 'n': int32 = 12345;
    std::print.stored[to.deci64['n'], newln];
    let 'big': int64 = 12345678901234567;
    let 'bd' := to.deci64['big'] otherwise 'price';
    std::print.stored['bd', newln];
    let 't' := to.deci64["1.250"] otherwise 'price';
    let 'u' := to.deci64["0.12345678901234567"] otherwise 'price';
    let 'v' := to.deci64["abc"] otherwise 'h';
    std::print.stored['t', " ", 'u', " ", 'v', newln];
    let 'i' := to.int32['h' times 2] otherwise -1;
    let 'j' := to.int32['h'] otherwise -1;
    std::print.stored['i', " ", 'j', " ", to.bin64['h'], " ", to.stored.str.utf8['h'], newln];
    let 'neg': deci64 = -1.5;
    std::print.stored['neg', " ", -'neg', " ", 'neg' < 'h', newln];
}
