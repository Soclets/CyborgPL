!MAIN:
    let $c: %channel[int.sign.32] := %channel.empty; %whichever:
        case $v from lend $c:
        end of case $v
    end of %whichever
!end of MAIN
