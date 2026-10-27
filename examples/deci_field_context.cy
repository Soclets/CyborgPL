# A literal takes its type from where it lands (types.md): a struct field's type and an enum case's data
# type are its context, as a declared variable's is, so `0.45` into a `d64` field is a deci.
record Price:
    amount: float.%decimal.64_bit;
end of record Price
union.tagged Pay:
    cash;
    voucher: float.%decimal.64_bit;
end of union.tagged Pay
!MAIN:
    let $p := Price[amount = 0.45];
    let $v := Pay.voucher(5.00);
    std::print.shortest($p.amount, " ", $v, newline);
!end of MAIN
