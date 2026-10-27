!MAIN:
    let $x := "a"; %spawn.task.detached with [lend $x]:
    end of %spawn.task.detached;
!end of MAIN
