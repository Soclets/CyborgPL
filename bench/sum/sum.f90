program sum_bench
  implicit none
  character(len=32) :: s
  integer :: st
  integer(8) :: n
  integer(8) :: i, acc
  call get_environment_variable("N", s, status=st)
  if (st /= 0) call exit(2)
  read(s, *, iostat=st) n
  if (st /= 0 .or. n < 0 .or. n > 1000000000_8) call exit(2)
  acc = 0
  do i = 1, n
    acc = acc + mod(i * i, 7_8)
  end do
  call exit(int(mod(mod(acc, 1000000007_8), 256_8)))
end program
