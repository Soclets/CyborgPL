
module text:
    %extern.C:
        pub proc strlen [let $s: ptr[int.unsign.8]] -> int.unsign.64;
        proc write [let $fd: int.sign.32, let $buf: ptr[int.unsign.8], let $len: int.unsign.64] -> int.sign.64;
    end of %extern.C
end of module text
!MAIN:
    let $msg := "hi";
    !no_check let $n := text.write(1, lend $msg, 2);
!end of MAIN
