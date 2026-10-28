union.tag Mood:
    calm;
    loud;
;union.tag Mood
fn say [let $m: Mood] -> i32:
    ret 1;
;fn say
MAIN:
    let $x := say(calm);
;MAIN
