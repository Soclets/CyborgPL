# Finding 2026-09-29 (check run, seed 6868, shrunk): a return inside a match arm of 'v9': u16 = match …
# lands in f3's return type (u64[min, 100]), but the oracle sizes its bare group by the declaration:
# "`shr` by 42 on a 16-bit value". CCA runs it.
# check claude generator, seed 6868
%extern.C fn write ['fd': i32, 'buf': ptr[u8], 'len': u64] -> i64;

fn emit ['m': u64, 'neg': bool] {
    let mut 'buf': std::list[u8] = [];
    if 'neg' { std::list.append(lend.mut 'buf', value: 45); }
    let mut 'v': u64 = 'm';
    let mut 'ds': std::list[u8] = [];
    std::list.append(lend.mut 'ds', value: to.u8(48 plus ('v' mod.floor 10)));
    change 'v' = 'v' div.floor 10;
    while 'v' > 0 {
        std::list.append(lend.mut 'ds', value: to.u8(48 plus ('v' mod.floor 10)));
        change 'v' = 'v' div.floor 10;
    }
    let mut 'k': u64 = std::list.length(lend 'ds');
    while 'k' at_least 1 {
        std::list.append(lend.mut 'buf', value: 'ds'[pos: 'k']);
        change 'k' = 'k' minus 1;
    }
    std::list.append(lend.mut 'buf', value: 10);
    no_check write(1, lend 'buf', std::list.length(lend 'buf'));
}

fn put_u ['n': u64] {
    emit('n', false);
}

fn put_i ['n': i64] {
    if 'n' at_least 0 {
        emit(to.u64('n'), false);
    } else {
        let 'up': i64 = -('n' plus 1);
        emit(to.u64('up') plus 1, true);
    }
}

enum E1 {
    c1: i8,
    c2,
    c3: i64,
    c4: u16
}

fn f3 [] -> u64[max: 100] {
    let 'v9': u16 = match E1.c2 {
        c1 'd6' { is ((0 times 16852) mod.floor 58) plus 651; }
        c2 {
            return ((13 plus (3628437621139716329 plus 1)) mod.floor ((8997704354644785536 shr 42) plus (4 plus 14))) mod.floor 101;
        }
        c3 'd7' { is ((to.u16('d7' mod.floor 65536) minus ((((to.u16('d7' mod.floor 65536) shr 3) mod.floor 4) pow (to.u16('d7' mod.floor 65536) mod.floor 11)) mod.floor 1)) mod.floor 58) plus 651; }
        c4 'd8' { is (('d8' times ('d8' div.floor 9)) mod.floor 58) plus 651; }
    };
    return 0;
}

MAIN {
    let 'v10': u64 = f3();
}
