# A length bound on text names its unit (decided 2026-09-22): text has three lengths.
procedure greet [lent $name: string.utf8[minimum: 1, maximum: 10 %grapheme_clusters]]:
    std::print.shortest("hi ", $name, newline);
end of procedure greet

!MAIN:
    let $thai: string.utf8[minimum: 1, maximum: 10 %grapheme_clusters] = "เกี่ยว";       # 4 graphemes (เ|กี่|ย|ว), 6 points, 18 bytes: fits
    let $ascii: string.utf8[minimum: 0, maximum: 5 bytes] = "hello";            # 5 bytes: fits
    greet(lend $thai);
    greet(lend $ascii);
    variable $line: string.utf8[minimum: 0, maximum: 20 bytes] = string.empty;
    change $line = $line concatenate "twelve chars";                      # 12 bytes: fits
    std::print.shortest(std::string.bytes(lend $line), newline);
    let $short := to.stored.string.utf8[minimum: 0, maximum: 3 bytes](12345) otherwise "?";     # into a bounded text: fails out_of_range
    std::print.shortest($short, newline);
!end of MAIN
