case.any;
# A file's mark covers every name written in it: functions, types, fields,
# cases, consts and variables (owner, 2026-10-06). examples/case_file.cpls
# without printing: 6 + 6 x 10 + 2 = 68.
extern.C fn exit ['code': int32];
const 'Limit' := 3;

struct Point {
    X: int32,
    Y: int32
}

enum Mood {
    Happy,
    Grumpy
}

fn Sum ['p': Point] -> int32 {
    return 'P'.x plus 'p'.y plus 'LIMIT';
}

fn Score ['m': Mood] -> int32 {
    return match 'M' { HAPPY { is 2; } grumpy { is 1; } };
}

MAIN {
    let 'pt' := POINT[x = 1, y = 2];
    let 'Total' := SUM[copy 'PT'] plus sum[copy 'Pt'] times 10;
    no_check my.EXIT['total' plus score[mood.happy]];
}
