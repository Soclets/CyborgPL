; Spacer's integer invariant for pick_int.smt2, read in 64-bit signed bitvectors and checked by Z3 and CVC5 (the
; ruling: a guess, never a proof): pick_at <= n, i > 0, and (pick_at > 0 or pick_minutes < 0). unsat = inductive,
; and the site holds where the walk ends. n is a list's length, at most 2^63 - 1.
(set-logic QF_BV)
(define-fun Inv ((i (_ BitVec 64)) (pa (_ BitVec 64)) (pm (_ BitVec 64)) (n (_ BitVec 64))) Bool
  (and (bvsle pa n) (bvsgt i (_ bv0 64)) (or (bvsgt pa (_ bv0 64)) (bvslt pm (_ bv0 64)))))
(declare-const i (_ BitVec 64)) (declare-const pa (_ BitVec 64)) (declare-const pm (_ BitVec 64))
(declare-const n (_ BitVec 64)) (declare-const r (_ BitVec 64))
(assert (bvsge n (_ bv0 64)))
; the walk's own `$i plus 1` has its overflow check, passed before the next trip (layer 3 assumes landings passed)
(define-fun step_ok ((i (_ BitVec 64))) Bool (bvslt i (_ bv9223372036854775807 64)))
(assert (bvsge r (bvneg (_ bv2147483648 64)))) (assert (bvsle r (_ bv2147483647 64)))
(assert (or
  ; it holds on entry
  (not (Inv (_ bv1 64) (_ bv0 64) (bvneg (_ bv1 64)) n))
  ; a trip that picks keeps it
  (and (Inv i pa pm n) (bvsle i n) (step_ok i) (or (bvslt pm (_ bv0 64)) (bvslt r pm)) (not (Inv (bvadd i (_ bv1 64)) i r n)))
  ; a trip that passes keeps it
  (and (Inv i pa pm n) (bvsle i n) (step_ok i) (not (Inv (bvadd i (_ bv1 64)) pa pm n)))
  ; where the walk ends, the position is in the list when one was picked
  (and (Inv i pa pm n) (bvsgt i n) (bvsge pm (_ bv0 64)) (or (bvslt pa (_ bv1 64)) (bvsgt pa n)))))
(check-sat)
