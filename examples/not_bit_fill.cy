# Finding 2026-10-05 (check run --borrow-wrong 1000 --avoid none, seeds 24107 and 24126, shrunk and
# cut down by hand): `simd.fill(value not.bit (5 plus $v17))` with a u16 'v17', for a simd[u16, count 8].
# The width comes from any operand whose type is known (owner: a, 2026-09-23), here 'v17', and the
# lanes are u16 besides. The oracle stops: "`not.bit` flips a type's bits, so it needs to know how
# many". With ('v17' plus 5) it runs, as it does for not.bit (5 plus 'v17') in a comparison or a
# print: inside simd.fill it looks at the left operand only. kayo runs it.
# check claude generator, seed 24126
%external.C procedure write [$fd: integer.signed.32_bit, $buf: pointer[integer.unsigned.8_bit], $len: integer.unsigned.64_bit] -> integer.signed.64_bit;

procedure emit [$m: integer.unsigned.64_bit, $neg: boolean]:
    variable $buf: std::list[integer.unsigned.8_bit] = [];
    if $neg:
        std::list.append(lend.mutable $buf, value: 45);
    end of if
    variable $v: integer.unsigned.64_bit = $m;
    variable $ds: std::list[integer.unsigned.8_bit] = [];
    std::list.append(lend.mutable $ds, value: to.integer.unsigned.8_bit(48 plus ($v modulo.floor 10)));
    change $v = $v divide.floor 10;
    while $v greater_than 0:
        std::list.append(lend.mutable $ds, value: to.integer.unsigned.8_bit(48 plus ($v modulo.floor 10)));
        change $v = $v divide.floor 10;
    end of while
    variable $k: integer.unsigned.64_bit = std::list.length(lend $ds);
    while $k at_least 1:
        std::list.append(lend.mutable $buf, value: $ds[position: $k]);
        change $k = $k minus 1;
    end of while
    std::list.append(lend.mutable $buf, value: 10);
    !no_check write(1, lend $buf, std::list.length(lend $buf));
end of procedure emit

procedure put_u [$n: integer.unsigned.64_bit]:
    emit($n, false);
end of procedure put_u

procedure put_i [$n: integer.signed.64_bit]:
    if $n less_than 0:
        emit(to.integer.unsigned.64_bit(0 minus ($n plus 1)) plus 1, true);
    end of if
    else:
        emit(to.integer.unsigned.64_bit($n), false);
    end of else
end of procedure put_i
!MAIN:
    let $v17: integer.unsigned.16_bit = 0;
    let $s21: simd[integer.unsigned.16_bit, count: 8] = simd.fill(value: not.bit (5 plus $v17));
    put_u(to.integer.unsigned.64_bit(simd.max($s21)));
!end of MAIN
