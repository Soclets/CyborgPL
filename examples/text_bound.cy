# A length bound on text names its unit (decided 2026-09-22): text has three lengths.
proc greet [lent $name: str.utf8[min: 1, max: 10 %clusters]]:
    std::println("hi ", $name);
;proc greet

MAIN:
    let $thai: str.utf8[min: 1, max: 10 %clusters] := "เกี่ยว";       # 4 graphemes (เ|กี่|ย|ว), 6 points, 18 bytes: fits
    let $ascii: str.utf8[min: 0, max: 5 bytes] := "hello";            # 5 bytes: fits
    greet(lend $thai);
    greet(lend $ascii);
    var $line: str.utf8[min: 0, max: 20 bytes] = str.empty;
    change $line ++= "twelve chars";                      # 12 bytes: fits
    std::println(std::str.bytes(lend $line));
    let $short := to.stored.str.utf8[min: 0, max: 3 bytes](12345) ?? "?";     # into a bounded text: fails out_of_range
    std::println($short);
;MAIN
