# A length bound on text names its unit (decided 2026-09-22): text has three lengths.
fn greet [lent $name: str.utf8[min: 1, max: 10 graphemes]] { std::print.useful("hi ", $name, newln); }

MAIN {
    let $thai: str.utf8[min: 1, max: 10 graphemes] = "เกี่ยว";       # 4 graphemes (เ|กี่|ย|ว), 6 points, 18 bytes: fits
    let $ascii: str.utf8[min: 0, max: 5 bytes] = "hello";            # 5 bytes: fits
    greet(lend $thai);
    greet(lend $ascii);
    let mut $line: str.utf8[min: 0, max: 20 bytes] = str.empty;
    change $line = $line concat "twelve chars";                      # 12 bytes: fits
    std::print.useful(std::str.bytes(lend $line), newln);
    let $short := to.stored.str.utf8[min: 0, max: 3 bytes](12345) otherwise "?";     # into a bounded text: fails out_of_range
    std::print.useful($short, newln);
}
