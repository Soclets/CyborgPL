program fib_bench
  implicit none
  character(len=32) :: s
  integer :: st
  integer(8) :: n
  call get_environment_variable("N", s, status=st)
  if (st /= 0) call exit(2)
  read(s, *, iostat=st) n
  if (st /= 0 .or. n < 0 .or. n > 92_8) call exit(2)
  call exit(int(mod(fib(n), 256_8)))
contains
  recursive function fib(k) result(r)
    integer(8), intent(in) :: k
    integer(8) :: r
    if (k < 2) then
      r = k
    else
      r = fib(k - 1) + fib(k - 2)
    end if
  end function
end program
