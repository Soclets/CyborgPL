; night_trains quickest_from: after `for $i, $s in lend $net.stations`, `$net.stations[pos: $pick_at]` when
; $pick_minutes is not negative. A station's minutes is any i32 (a map value). Integers.
(set-logic HORN)
(declare-fun Inv (Int Int Int Int) Bool) ; i, pick_at, pick_minutes, n
(assert (forall ((n Int)) (=> (>= n 0) (Inv 1 0 (- 1) n))))
; a station picked: its position and minutes taken
(assert (forall ((i Int) (pa Int) (pm Int) (n Int) (r Int))
  (=> (and (and (Inv i pa pm n) (<= i n) (>= r (- 2147483648)) (<= r 2147483647)) (or (< pm 0) (< r pm))) (Inv (+ i 1) i r n))))
; a station passed over: nothing taken
(assert (forall ((i Int) (pa Int) (pm Int) (n Int) (r Int))
  (=> (and (Inv i pa pm n) (<= i n) (>= r (- 2147483648)) (<= r 2147483647)) (Inv (+ i 1) pa pm n))))
(assert (forall ((i Int) (pa Int) (pm Int) (n Int))
  (=> (and (Inv i pa pm n) (> i n) (>= pm 0) (or (< pa 1) (> pa n))) false)))
(check-sat)
(get-model)
