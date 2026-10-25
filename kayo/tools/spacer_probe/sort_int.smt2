; night_trains soonest_first: after appending, `$i := length`, then `while $i > 1 and.then $out[pos: $i] …
; $out[pos: $i - 1]`, swapping (the length does not change), `$i = $i - 1`. Integers.
(set-logic HORN)
(declare-fun Inv (Int Int) Bool) ; i, len
(assert (forall ((len Int)) (=> (>= len 1) (Inv len len))))
(assert (forall ((i Int) (len Int) (go Bool)) (=> (and (Inv i len) (> i 1) go) (Inv (- i 1) len))))
(assert (forall ((i Int) (len Int)) (=> (and (Inv i len) (> i 1) (or (< i 1) (> i len) (< (- i 1) 1) (> (- i 1) len))) false)))
(check-sat)
(get-model)
