; std str.cy sits_at: `for $k [from: $pos, to: $pos + $m - 1]`, reading $text[pos: $k] and $part[pos: $k - $pos + 1]
; after `if $m > $n` and `if $pos > $n - $m + 1` returned. Integers (u64 values are below 2^64).
(set-logic HORN)
(declare-fun Inv (Int Int Int Int) Bool) ; k, pos, n, m
(assert (forall ((pos Int) (n Int) (m Int))
  (=> (and (>= pos 1) (>= n 0) (>= m 0) (<= m n) (<= pos (+ (- n m) 1)) (< n 18446744073709551616)) (Inv pos pos n m))))
(assert (forall ((k Int) (pos Int) (n Int) (m Int))
  (=> (and (Inv k pos n m) (<= k (- (+ pos m) 1))) (Inv (+ k 1) pos n m))))
(assert (forall ((k Int) (pos Int) (n Int) (m Int))
  (=> (and (Inv k pos n m) (<= k (- (+ pos m) 1)) (or (< k 1) (> k n) (< (+ (- k pos) 1) 1) (> (+ (- k pos) 1) m))) false)))
(check-sat)
(get-model)
