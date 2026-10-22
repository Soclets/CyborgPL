# Two flags are two graphemes of two code points each (check claude, 2026-10-20): "🇹🇭🇯🇵!" is 3
# graphemes and 5 points, so a grapheme slice to position 4 runs past the end and fails, even when
# Precog knows the text. 3 graphemes x 10 + the fallback's 0 bytes: exits 30.
extern.C fn exit ['code': i32];
MAIN {
    let 's': str.utf8 = "🇹🇭🇯🇵!";
    let 'a' := std::str.slice.grapheme[lend 's', from: pos 2, to: pos 4] otherwise str.empty;
    no_check my.exit[to.i32[std::str.graphemes[lend 's'] x 10 + std::str.bytes[lend 'a']]];
}
