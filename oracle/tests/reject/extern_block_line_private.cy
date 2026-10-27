
module text:
    %extern.C:
        pub proc strlen [$s: ptr[int.unsign.8]] -> int.unsign.64;
        proc write [$fd: int.sign.32, $buf: ptr[int.unsign.8], $len: int.unsign.64] -> int.sign.64;
    end of %extern.C
end of module text
!MAIN:
    let $msg := "hi";
    !no_check let $n := text.write(1, lend $msg, 2);
!end of MAIN
