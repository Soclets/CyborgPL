fn f [] -> int.sign.32 or fail[bad]:
    fail bad;
end of fn f
!MAIN:
    %spawn.task:
        let $v := f() otherwise fail;
    end of %spawn.task;
!end of MAIN
