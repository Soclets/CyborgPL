; The same as sits_int.smt2, in 64-bit bitvectors (unsigned, the program's u64), as layer 3 encodes.
(set-logic HORN)
(declare-fun Inv ((_ BitVec 64) (_ BitVec 64) (_ BitVec 64) (_ BitVec 64)) Bool) ; k, pos, n, m
(assert (forall ((pos (_ BitVec 64)) (n (_ BitVec 64)) (m (_ BitVec 64)))
  (=> (and (bvuge pos (_ bv1 64)) (bvule m n) (bvule pos (bvadd (bvsub n m) (_ bv1 64)))) (Inv pos pos n m))))
(assert (forall ((k (_ BitVec 64)) (pos (_ BitVec 64)) (n (_ BitVec 64)) (m (_ BitVec 64)))
  (=> (and (Inv k pos n m) (bvule k (bvsub (bvadd pos m) (_ bv1 64)))) (Inv (bvadd k (_ bv1 64)) pos n m))))
(assert (forall ((k (_ BitVec 64)) (pos (_ BitVec 64)) (n (_ BitVec 64)) (m (_ BitVec 64)))
  (=> (and (Inv k pos n m) (bvule k (bvsub (bvadd pos m) (_ bv1 64)))
           (or (bvult k (_ bv1 64)) (bvugt k n) (bvult (bvadd (bvsub k pos) (_ bv1 64)) (_ bv1 64)) (bvugt (bvadd (bvsub k pos) (_ bv1 64)) m))) false)))
(check-sat)
(get-model)
