%extern.C proc exit [let $code: i32];
fn judge [let $n: i32] -> fail[small, odd: i32]:
    if $n < 10:
        fail small;
    ;if
    if ($n mod.floor 2) == 1:
        fail odd[$n];
    ;if
;fn judge
MAIN:
    var $seen: i32 = 0;
    block:
        %spawn.task with []:
            judge(3)?;
        ;%spawn.task
            otherwise:
                case small:
                    change $seen to $seen + 1;
                ;case small
                case odd $v:
                    change $seen to $seen + 100;
                ;case odd
            ;otherwise;
        %spawn.task with []:
            judge(13)?;
        ;%spawn.task
            otherwise:
                case small:
                    change $seen to $seen + 1000;
                ;case small
                case odd $v:
                    change $seen to $seen + $v;
                ;case odd
            ;otherwise;
        %spawn.task with []:
            judge(20)?;
        ;%spawn.task
            otherwise:
                case small:
                    change $seen to $seen + 7;
                ;case small
                case odd $v:
                    change $seen to $seen + 7;
                ;case odd
            ;otherwise;
    ;block
    no_check my.exit($seen);
;MAIN
