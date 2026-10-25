; The same as sort_int.smt2, in 64-bit bitvectors (unsigned, a length and a position), as layer 3 encodes.
(set-logic HORN)
(declare-fun Inv ((_ BitVec 64) (_ BitVec 64)) Bool) ; i, len
(assert (forall ((len (_ BitVec 64))) (=> (bvuge len (_ bv1 64)) (Inv len len))))
(assert (forall ((i (_ BitVec 64)) (len (_ BitVec 64)) (go Bool)) (=> (and (Inv i len) (bvugt i (_ bv1 64)) go) (Inv (bvsub i (_ bv1 64)) len))))
(assert (forall ((i (_ BitVec 64)) (len (_ BitVec 64))) (=> (and (Inv i len) (bvugt i (_ bv1 64)) (or (bvult i (_ bv1 64)) (bvugt i len) (bvult (bvsub i (_ bv1 64)) (_ bv1 64)) (bvugt (bvsub i (_ bv1 64)) len))) false)))
(check-sat)
(get-model)
