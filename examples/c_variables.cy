# C's variables and pointers holding a C enum (owner, 2026-09-28: b). A C variable holds C's
# plain number, since reading a variable never fails; convert it with `to.Mode(…)`, and back
# with `to.i32(Mode.vi)`, which never fails. `ptr.read` of a C enum is a call, so it takes
# `otherwise` and fails `out_of_range` for a number no case has (7 here).
%layout.C enumeration Mode:
    emacs = 0, vi = 1
end of layout
%external.C procedure malloc [$bytes: integer.unsigned.64_bit] -> pointer.mutable[Mode];
%external.C variable $edit_mode: integer.signed.32_bit;
procedure mode_at [$p: pointer[Mode]] -> integer.signed.32_bit:
    !no_check let $m := pointer.read($p) otherwise Mode.emacs;
    match $m:
        case emacs:
            return 10;
        end of case emacs
        case vi:
            return 11;
        end of case vi
    end of match
end of procedure mode_at
!MAIN:
    !no_check change $edit_mode = to.integer.signed.32_bit(Mode.vi);
    !no_check let $raw := $edit_mode;
    let $current := to.Mode($raw) otherwise Mode.emacs;
    !no_check let $mem := malloc(8);
    !no_check pointer.write($mem, value: 1);
    !no_check pointer.write(pointer.at($mem, by: 1), value: 7);
    !no_check let $a := mode_at($mem);
    !no_check let $b := mode_at(pointer.at($mem, by: 1));
    std::print.shortest(to.integer.signed.32_bit($current), " ", $a, " ", $b, newline);
!end of MAIN
