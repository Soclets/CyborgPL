program sieve_bench
  implicit none
  character(len=32) :: s
  integer :: st
  integer(8) :: n
  logical, allocatable :: comp(:)
  integer(8) :: p, m, count
  call get_environment_variable("N", s, status=st)
  if (st /= 0) call exit(2)
  read(s, *, iostat=st) n
  if (st /= 0 .or. n < 0 .or. n > 200000000_8) call exit(2)
  allocate(comp(0:n))
  comp = .false.
  count = 0
  do p = 2, n
    if (.not. comp(p)) then
      count = count + 1
      m = p * p
      do while (m <= n)
        comp(m) = .true.
        m = m + p
      end do
    end if
  end do
  call exit(int(mod(count, 256_8)))
end program
