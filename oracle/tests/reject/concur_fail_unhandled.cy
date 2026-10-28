fn f [] -> i32 or fail[bad]:
    fail bad;
;fn f
MAIN:
    %spawn.task:
        let $v := f()?;
    ;%spawn.task;
;MAIN
