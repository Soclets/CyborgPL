!MAIN:
    let $c: %channel[integer.signed.32_bit] = %channel.empty; %whichever:
        case $v from lend $c:
        end of case $v
    end of %whichever
!end of MAIN
