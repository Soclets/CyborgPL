# A value a function built and handed back is given back by whoever holds
# it — and a fresh one sent, appended or set is moved in, never given back
# twice. Both were wrong until 2026-09-29: an answer held in a variable was
# never freed, which only hid the second.
%extern.C proc exit [let $code: i32];
fn page [lent $u: str.utf8] -> str.utf8:
    ret "page of " concat $u;
;fn page
fn run [] -> i32:
    var $xs: std::ls[str.utf8] = [];
    let $a := "a";
    std::ls.append(lend.mut $xs, value: page(lend $a));
    std::ls.append(lend.mut $xs, value: page("b"));
    var $m: std::map[str.utf8, str.utf8] = std::map.empty;
    std::map.set(lend.mut $m, key: page("k"), value: page("v"));
    ret to.i32(std::ls.len(lend $xs)) + to.i32(std::map.count(lend $m));
;fn run
MAIN:
    no_check my.exit(run());
;MAIN
