union.tag Mood:
    calm;
    loud;
end of union.tag Mood
fn say [$m: Mood] -> int.sign.32:
    ret 1;
end of fn say
!MAIN:
    let $x := say(calm);
!end of MAIN
