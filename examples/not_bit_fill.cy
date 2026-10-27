# Finding 2026-10-05 (check run --borrow-wrong 1000 --avoid none, seeds 24107 and 24126, shrunk and
# cut down by hand): `simd.fill(value not.bit (5 plus $v17))` with a u16 'v17', for a simd[u16, count 8].
# The width comes from any operand whose type is known (owner: a, 2026-09-23), here 'v17', and the
# lanes are u16 besides. The oracle stops: "`not.bit` flips a type's bits, so it needs to know how
# many". With ('v17' plus 5) it runs, as it does for not.bit (5 plus 'v17') in a comparison or a
# print: inside simd.fill it looks at the left operand only. kayo runs it.
# check claude generator, seed 24126
%extern.C proc write [let $fd: int.sign.32, let $buf: ptr[int.unsign.8], let $len: int.unsign.64] -> int.sign.64;

proc emit [let $m: int.unsign.64, let $neg: bool]:
    var $buf: std::ls[int.unsign.8] = [];
    if $neg:
        std::ls.append(lend.mut $buf, value: 45);
    end of if
    var $v: int.unsign.64 = $m;
    var $ds: std::ls[int.unsign.8] = [];
    std::ls.append(lend.mut $ds, value: to.int.unsign.8(48 plus ($v mod.floor 10)));
    change $v = $v div.floor 10;
    while $v greater_than 0:
        std::ls.append(lend.mut $ds, value: to.int.unsign.8(48 plus ($v mod.floor 10)));
        change $v = $v div.floor 10;
    end of while
    var $k: int.unsign.64 = std::ls.len(lend $ds);
    while $k at_least 1:
        std::ls.append(lend.mut $buf, value: $ds[pos: $k]);
        change $k = $k minus 1;
    end of while
    std::ls.append(lend.mut $buf, value: 10);
    !no_check write(1, lend $buf, std::ls.len(lend $buf));
end of proc emit

proc put_u [let $n: int.unsign.64]:
    emit($n, false);
end of proc put_u

proc put_i [let $n: int.sign.64]:
    if $n less_than 0:
        emit(to.int.unsign.64(0 minus ($n plus 1)) plus 1, true);
    end of if
    else:
        emit(to.int.unsign.64($n), false);
    end of else
end of proc put_i
!MAIN:
    let $v17: int.unsign.16 = 0;
    let $s21: simd[int.unsign.16, count: 8] = simd.fill(value: not.bit (5 plus $v17));
    put_u(to.int.unsign.64(simd.max($s21)));
!end of MAIN
