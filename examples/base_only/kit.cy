# No std listed: no operators and no printing (owner, 2026-10-06). Functions, `let`, `if`,
# structs, `match`, and.then / or.else (the base's, since they skip their right side) still work.

record Point:
    x: integer.signed.32_bit;
    y: integer.signed.32_bit;
end of record Point

union.tagged Light:
    lit;
    dark;
end of union.tagged Light

function ready [] -> boolean:
    return true;
end of function ready

function origin [] -> Point:
    return Point[x = 0, y = 0];
end of function origin

!MAIN:
    let $p := origin();
    let $l := Light.lit;
    if ready() and.then false:
        let $unused: integer.signed.32_bit = 1;
    end of if
    match $l:
        case lit:
        end of case lit
        case dark:
        end of case dark
    end of match
!end of MAIN

# a fixed array and a text literal are the base's too (owner, 2026-10-06)
function greeting [] -> string.utf8:
    return "hi";
end of function greeting

function three [] -> array[integer.signed.32_bit]:
    return [1, 2, 3,];
end of function three

# std's `exit` is out of reach without std, so this kit's own `exit` is the only one (2026-10-06)
function exit []:
end of function exit

procedure stop []:
    exit();
end of procedure stop
