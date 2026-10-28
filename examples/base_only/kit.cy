# No std listed: no operators and no printing (owner, 2026-10-06). Functions, `let`, `if`,
# structs, `match`, and.then / or.else (the base's, since they skip their right side) still work.

rec Point:
    var x: i32;
    var y: i32;
;rec Point

union.tag Light:
    lit;
    dark;
;union.tag Light

fn ready [] -> bool:
    ret true;
;fn ready

fn origin [] -> Point:
    ret Point[x = 0, y = 0];
;fn origin

MAIN:
    let $p := origin();
    let $l := Light.lit;
    if ready() and.then false:
        let $unused: i32 := 1;
    ;if
    match $l:
        case lit:
        ;case lit
        case dark:
        ;case dark
    ;match
;MAIN

# a fixed array and a text literal are the base's too (owner, 2026-10-06)
fn greeting [] -> str.utf8:
    ret "hi";
;fn greeting

fn three [] -> arr[i32]:
    ret [1, 2, 3,];
;fn three

# std's `exit` is out of reach without std, so this kit's own `exit` is the only one (2026-10-06)
fn exit []:
;fn exit

proc stop []:
    exit();
;proc stop
