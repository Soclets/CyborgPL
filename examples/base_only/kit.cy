# No std listed: no operators and no printing (owner, 2026-10-06). Functions, `let`, `if`,
# structs, `match`, and.then / or.else (the base's, since they skip their right side) still work.

rec Point:
    var x: int.sign.32;
    var y: int.sign.32;
end of rec Point

union.tag Light:
    lit;
    dark;
end of union.tag Light

fn ready [] -> bool:
    ret true;
end of fn ready

fn origin [] -> Point:
    ret Point[x = 0, y = 0];
end of fn origin

!MAIN:
    let $p := origin();
    let $l := Light.lit;
    if ready() and.then false:
        let $unused: int.sign.32 = 1;
    end of if
    match $l:
        case lit:
        end of case lit
        case dark:
        end of case dark
    end of match
!end of MAIN

# a fixed array and a text literal are the base's too (owner, 2026-10-06)
fn greeting [] -> str.utf8:
    ret "hi";
end of fn greeting

fn three [] -> arr[int.sign.32]:
    ret [1, 2, 3,];
end of fn three

# std's `exit` is out of reach without std, so this kit's own `exit` is the only one (2026-10-06)
fn exit []:
end of fn exit

proc stop []:
    exit();
end of proc stop
