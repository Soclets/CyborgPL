!MAIN:
    variable $xs := [1, 2, 3];          # a list of integers, though nobody wrote the type
    let $x: float.binary.32_bit = 70.0;
    change $xs[position: 1] = $x divide.nearest 2.0;        # a bin into an integer element: a family crossing
!end of MAIN
