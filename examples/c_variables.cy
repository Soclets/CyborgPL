# C's variables and pointers holding a C enum (owner, 2026-09-28: b). A C variable holds C's
# plain number, since reading a variable never fails; convert it with `to.Mode(…)`, and back
# with `to.i32(Mode.vi)`, which never fails. `ptr.read` of a C enum is a call, so it takes
# `otherwise` and fails `out_of_range` for a number no case has (7 here).
%layout.C enum Mode:
    emacs = 0;
    vi = 1;
end of enum Mode
%extern.C proc malloc [let $bytes: int.unsign.64] -> ptr.mut[Mode];
%extern.C var $edit_mode: int.sign.32;
proc mode_at [let $p: ptr[Mode]] -> int.sign.32:
    !no_check let $m := ptr.read($p) otherwise Mode.emacs;
    match $m:
        case emacs:
            ret 10;
        end of case emacs
        case vi:
            ret 11;
        end of case vi
    end of match
end of proc mode_at
!MAIN:
    !no_check change $edit_mode to to.int.sign.32(Mode.vi);
    !no_check let $raw := $edit_mode;
    let $current := to.Mode($raw) otherwise Mode.emacs;
    !no_check let $mem := malloc(8);
    !no_check ptr.write($mem, value: 1);
    !no_check ptr.write(ptr.at($mem, by: 1), value: 7);
    !no_check let $a := mode_at($mem);
    !no_check let $b := mode_at(ptr.at($mem, by: 1));
    std::print.shortest(to.int.sign.32($current), " ", $a, " ", $b, newln);
!end of MAIN
