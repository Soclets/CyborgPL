union.tagged Mood:
    calm; loud;
end of union.tagged Mood
function say [$m: Mood] -> integer.signed.32_bit:
    return 1;
end of function say
!MAIN:
    let $x := say(calm);
!end of MAIN
