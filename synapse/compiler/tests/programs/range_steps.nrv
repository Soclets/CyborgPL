# A step past the counter's last value is never taken (control-flow.md,
# owner 2026-09-23), and a negative step counts a uint64 down.
extern.C fn exit ['code': int32];
MAIN {
    let mut 't': int32 = 0;
    for 'i': uint8 from 1 to 0 by -2 { change 't' = 't' plus 1; }
    for 'i': uint8 from 250 to 255 by 3 { change 't' = 't' plus 1; }
    for 'i': int8 from 120 to 127 by 5 { change 't' = 't' plus 1; }
    for 'i': int8 from -120 to -128 by -5 { change 't' = 't' plus 1; }
    for 'i': uint64 from 3 to 1 by -1 { change 't' = 't' plus 1; }
    for 'i': int64 from 9223372036854775806 to 9223372036854775807 by 2 { change 't' = 't' plus 1; }
    no_check my.exit['t'];
}
