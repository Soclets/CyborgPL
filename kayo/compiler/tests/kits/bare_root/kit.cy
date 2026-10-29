# 3 plus 4, worked in `adds`, which lists std; this Kit does not.
%extern.C proc exit [let $code: i32];
MAIN:
    no_check my.exit(adds::sum(3, 4));
;MAIN
