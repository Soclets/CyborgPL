function f [] -> integer.signed.32_bit or fail[bad]:
    fail bad;
end of function f
!MAIN:
    %spawn.task:
        let $v := f() otherwise fail;
    end of %spawn.task;
!end of MAIN
