
module text:
    %external.C:
        public procedure strlen [$s: pointer[integer.unsigned.8_bit]] -> integer.unsigned.64_bit;
        procedure write [$fd: integer.signed.32_bit, $buf: pointer[integer.unsigned.8_bit], $len: integer.unsigned.64_bit] -> integer.signed.64_bit;
    end of %external.C
end of module text
!MAIN:
    let $msg := "hi";
    !no_check let $n := text.write(1, lend $msg, 2);
!end of MAIN
