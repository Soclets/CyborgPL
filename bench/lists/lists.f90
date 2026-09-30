program lists_bench
  implicit none
  character(len=32) :: s
  integer :: st
  integer(8) :: n
  integer(8), allocatable :: xs(:), tmp(:)
  integer(8) :: i, j, acc, length, cap
  integer :: r
  call get_environment_variable("N", s, status=st)
  if (st /= 0) call exit(2)
  read(s, *, iostat=st) n
  if (st /= 0 .or. n < 0 .or. n > 100000000_8) call exit(2)
  cap = 4
  length = 0
  allocate(xs(cap))
  do i = 1, n
    if (length == cap) then
      allocate(tmp(cap * 2))
      tmp(1:length) = xs(1:length)
      call move_alloc(tmp, xs)
      cap = cap * 2
    end if
    length = length + 1
    xs(length) = mod(i, 1000_8)
  end do
  acc = 0
  do r = 1, 10
    do j = 1, length
      acc = acc + xs(j)
    end do
  end do
  call exit(int(mod(acc, 256_8)))
end program
