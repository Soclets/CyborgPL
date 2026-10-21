! n-body, after the Computer Language Benchmarks Game: the Sun and the four Jovian planets, N steps of
! the symplectic integrator with dt = 0.01, the energy printed to 9 places before and after. N comes
! from the environment and must be 0 to 1_000_000_000; anything else ends the program with exit code 2.
! Every version in bench/nbody/ does this arithmetic in this order, so their energies agree to the last
! digit; none fuses a multiply and an add (built with -ffp-contract=off).
program nbody
  implicit none
  integer, parameter :: dp = kind(1.0d0)
  integer, parameter :: n_bodies = 5
  real(dp), parameter :: pi = 3.141592653589793_dp
  real(dp), parameter :: solar_mass = 4.0_dp * pi * pi
  real(dp), parameter :: days_per_year = 365.24_dp

  type body
    real(dp) :: x, y, z, vx, vy, vz, mass
  end type body

  character(len=32) :: s
  integer :: st
  integer(8) :: n, step
  type(body) :: bodies(n_bodies)
  call get_environment_variable("N", s, status=st)
  if (st /= 0) call exit(2)
  read(s, *, iostat=st) n
  if (st /= 0 .or. n < 0 .or. n > 1000000000_8) call exit(2)
  ! the Sun
  bodies(1) = body(0.0_dp, 0.0_dp, 0.0_dp, 0.0_dp, 0.0_dp, 0.0_dp, solar_mass)
  ! Jupiter
  bodies(2) = body(4.84143144246472090e+00_dp, -1.16032004402742839e+00_dp, -1.03622044471123109e-01_dp, &
                   1.66007664274403694e-03_dp * days_per_year, 7.69901118419740425e-03_dp * days_per_year, &
                   -6.90460016972063023e-05_dp * days_per_year, 9.54791938424326609e-04_dp * solar_mass)
  ! Saturn
  bodies(3) = body(8.34336671824457987e+00_dp, 4.12479856412430479e+00_dp, -4.03523417114321381e-01_dp, &
                   -2.76742510726862411e-03_dp * days_per_year, 4.99852801234917238e-03_dp * days_per_year, &
                   2.30417297573763929e-05_dp * days_per_year, 2.85885980666130812e-04_dp * solar_mass)
  ! Uranus
  bodies(4) = body(1.28943695621391310e+01_dp, -1.51111514016986312e+01_dp, -2.23307578892655734e-01_dp, &
                   2.96460137564761618e-03_dp * days_per_year, 2.37847173959480950e-03_dp * days_per_year, &
                   -2.96589568540237556e-05_dp * days_per_year, 4.36624404335156298e-05_dp * solar_mass)
  ! Neptune
  bodies(5) = body(1.53796971148509165e+01_dp, -2.59193146099879641e+01_dp, 1.79258772950371181e-01_dp, &
                   2.68067772490389322e-03_dp * days_per_year, 1.62824170038242295e-03_dp * days_per_year, &
                   -9.51592254519715870e-05_dp * days_per_year, 5.15138902046611451e-05_dp * solar_mass)
  call offset_momentum(bodies)
  write (*, '(F12.9)') energy(bodies)
  do step = 1, n
    call advance(bodies, 0.01_dp)
  end do
  write (*, '(F12.9)') energy(bodies)

contains

  subroutine advance(bodies, dt)
    type(body), intent(inout) :: bodies(n_bodies)
    real(dp), intent(in) :: dt
    integer :: i, j
    real(dp) :: dx, dy, dz, distance, mag
    do i = 1, n_bodies
      do j = i + 1, n_bodies
        dx = bodies(i)%x - bodies(j)%x
        dy = bodies(i)%y - bodies(j)%y
        dz = bodies(i)%z - bodies(j)%z
        distance = sqrt(dx * dx + dy * dy + dz * dz)
        mag = dt / (distance * distance * distance)
        bodies(i)%vx = bodies(i)%vx - dx * bodies(j)%mass * mag
        bodies(i)%vy = bodies(i)%vy - dy * bodies(j)%mass * mag
        bodies(i)%vz = bodies(i)%vz - dz * bodies(j)%mass * mag
        bodies(j)%vx = bodies(j)%vx + dx * bodies(i)%mass * mag
        bodies(j)%vy = bodies(j)%vy + dy * bodies(i)%mass * mag
        bodies(j)%vz = bodies(j)%vz + dz * bodies(i)%mass * mag
      end do
    end do
    do i = 1, n_bodies
      bodies(i)%x = bodies(i)%x + dt * bodies(i)%vx
      bodies(i)%y = bodies(i)%y + dt * bodies(i)%vy
      bodies(i)%z = bodies(i)%z + dt * bodies(i)%vz
    end do
  end subroutine advance

  function energy(bodies) result(e)
    type(body), intent(in) :: bodies(n_bodies)
    real(dp) :: e, dx, dy, dz, distance
    integer :: i, j
    e = 0.0_dp
    do i = 1, n_bodies
      e = e + 0.5_dp * bodies(i)%mass * (bodies(i)%vx * bodies(i)%vx + bodies(i)%vy * bodies(i)%vy &
                                          + bodies(i)%vz * bodies(i)%vz)
      do j = i + 1, n_bodies
        dx = bodies(i)%x - bodies(j)%x
        dy = bodies(i)%y - bodies(j)%y
        dz = bodies(i)%z - bodies(j)%z
        distance = sqrt(dx * dx + dy * dy + dz * dz)
        e = e - (bodies(i)%mass * bodies(j)%mass) / distance
      end do
    end do
  end function energy

  subroutine offset_momentum(bodies)
    type(body), intent(inout) :: bodies(n_bodies)
    real(dp) :: px, py, pz
    integer :: i
    px = 0.0_dp
    py = 0.0_dp
    pz = 0.0_dp
    do i = 1, n_bodies
      px = px + bodies(i)%vx * bodies(i)%mass
      py = py + bodies(i)%vy * bodies(i)%mass
      pz = pz + bodies(i)%vz * bodies(i)%mass
    end do
    bodies(1)%vx = (-px) / solar_mass
    bodies(1)%vy = (-py) / solar_mass
    bodies(1)%vz = (-pz) / solar_mass
  end subroutine offset_momentum

end program nbody
