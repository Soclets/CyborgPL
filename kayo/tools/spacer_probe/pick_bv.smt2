; The same as pick_int.smt2, in 64-bit bitvectors (signed), as layer 3 encodes.
(set-logic HORN)
(declare-fun Inv ((_ BitVec 64) (_ BitVec 64) (_ BitVec 64) (_ BitVec 64)) Bool) ; i, pick_at, pick_minutes, n
(assert (forall ((n (_ BitVec 64))) (=> (bvsge n (_ bv0 64)) (Inv (_ bv1 64) (_ bv0 64) (bvneg (_ bv1 64)) n))))
; a station picked: its position and minutes taken
(assert (forall ((i (_ BitVec 64)) (pa (_ BitVec 64)) (pm (_ BitVec 64)) (n (_ BitVec 64)) (r (_ BitVec 64)))
  (=> (and (and (Inv i pa pm n) (bvsle i n) (bvsge r (bvneg (_ bv2147483648 64))) (bvsle r (_ bv2147483647 64))) (or (bvslt pm (_ bv0 64)) (bvslt r pm))) (Inv (bvadd i (_ bv1 64)) i r n))))
; a station passed over: nothing taken
(assert (forall ((i (_ BitVec 64)) (pa (_ BitVec 64)) (pm (_ BitVec 64)) (n (_ BitVec 64)) (r (_ BitVec 64)))
  (=> (and (Inv i pa pm n) (bvsle i n) (bvsge r (bvneg (_ bv2147483648 64))) (bvsle r (_ bv2147483647 64))) (Inv (bvadd i (_ bv1 64)) pa pm n))))
(assert (forall ((i (_ BitVec 64)) (pa (_ BitVec 64)) (pm (_ BitVec 64)) (n (_ BitVec 64)))
  (=> (and (Inv i pa pm n) (bvsgt i n) (bvsge pm (_ bv0 64)) (or (bvslt pa (_ bv1 64)) (bvsgt pa n))) false)))
(check-sat)
(get-model)
