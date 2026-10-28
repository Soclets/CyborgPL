# A literal takes its type from where it lands (types.md): a struct field's type and an enum case's data
# type are its context, as a declared variable's is, so `0.45` into a `d64` field is a deci.
rec Price:
    var amount: %d64;
;rec Price
union.tag Pay:
    cash;
    voucher: %d64;
;union.tag Pay
MAIN:
    let $p := Price[amount = 0.45];
    let $v := Pay.voucher(5.00);
    std::print($p.amount, " ", $v, newln);
;MAIN
