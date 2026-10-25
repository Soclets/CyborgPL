; Spacer's integer invariant for sits_int.smt2 (k >= pos, k > 0, pos - n + m <= 1), read in 64-bit unsigned
; bitvectors and checked by Z3 and CVC5. unsat = inductive, and both positions are in their lists.
(set-logic QF_BV)
(define-fun Inv ((k (_ BitVec 64)) (pos (_ BitVec 64)) (n (_ BitVec 64)) (m (_ BitVec 64))) Bool
  (and (bvuge k pos) (bvugt k (_ bv0 64)) (bvule pos (bvadd (bvsub n m) (_ bv1 64))) (bvule m n) (bvuge pos (_ bv1 64))))
; `$pos + $m - 1` and the loop's `$k plus 1` have their overflow checks, passed (layer 3 assumes landings passed)
(define-fun no_wrap ((a (_ BitVec 64)) (b (_ BitVec 64))) Bool (bvule a (bvsub (_ bv18446744073709551615 64) b)))
(declare-const k (_ BitVec 64)) (declare-const pos (_ BitVec 64)) (declare-const n (_ BitVec 64)) (declare-const m (_ BitVec 64))
(assert (and (no_wrap pos m) (bvuge (bvadd pos m) (_ bv1 64))))
(assert (or
  (and (bvuge pos (_ bv1 64)) (bvule m n) (bvule pos (bvadd (bvsub n m) (_ bv1 64))) (not (Inv pos pos n m)))
  (and (Inv k pos n m) (bvule k (bvsub (bvadd pos m) (_ bv1 64))) (no_wrap k (_ bv1 64)) (not (Inv (bvadd k (_ bv1 64)) pos n m)))
  (and (Inv k pos n m) (bvule k (bvsub (bvadd pos m) (_ bv1 64)))
       (or (bvult k (_ bv1 64)) (bvugt k n) (bvult (bvadd (bvsub k pos) (_ bv1 64)) (_ bv1 64)) (bvugt (bvadd (bvsub k pos) (_ bv1 64)) m)))))
(check-sat)
