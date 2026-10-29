# A C variable holds C's plain number (owner, 2026-09-28: b): it converts to
# a C enum with `to.Mode[…]` and back with `to.i32[Mode.vi]`, and a
# `ptr.read` of a C enum fails `out_of_range` for a number no case has
# (examples/c_variables.cpls, with libc's own `optind` as the C variable, so
# the oracle cannot link it and the answer is worked out by hand: 1, 11, 10
# → 100 + 10 + 0 = 110).
%extern.C proc exit [let $code: i32];
%extern.C proc malloc [let $bytes: u64] -> ptr.mut[Mode];
%extern.C proc free [let $p: ptr.mut[Mode]];
%extern.C var $optind: i32;
%layout.C enum Mode:
    emacs = 0; vi = 1;
;%layout.C enum Mode
proc mode_at [let $p: ptr[Mode]] -> i32:
    no_check let $m := ptr.read($p) ?? Mode.emacs;
    match $m:
        case emacs:
            ret 10;
        ;case emacs
        case vi:
            ret 11;
        ;case vi
    ;match
;proc mode_at
MAIN:
    no_check change $optind to to.i32(Mode.vi);
    no_check let $raw := $optind;
    let $current := to.Mode($raw) ?? Mode.emacs;
    no_check let $mem := malloc(8);
    no_check ptr.write($mem, value: 1);
    no_check ptr.write(ptr.at($mem, by: 1), value: 7);
    no_check let $a := mode_at($mem);
    no_check let $b := mode_at(ptr.at($mem, by: 1));
    no_check free($mem);
    no_check my.exit(to.i32($current) x 100 + ($a - 10) x 10 + ($b - 10));
;MAIN
