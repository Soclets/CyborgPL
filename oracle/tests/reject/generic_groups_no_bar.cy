function f <T: any> @x [lent @x $a: std::list[<T>]] -> lent @x std::list[<T>]:
    return lend $a;
end of function f
!MAIN:
!end of MAIN
