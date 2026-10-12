/* For examples/c_types.nrv: enums, a function pointer, and C's machine-sized types. */
#include <stddef.h>
#include <stdint.h>
enum color { RED, GREEN = 5, BLUE };
typedef enum { LOW = -1, HIGH } level_t;
enum { SEEK_A = 0, SEEK_B = 2 };
void paint(enum color c, level_t l);
void sort(void *b, unsigned long n, int (*cmp)(const void *, const void *));
long count(const char *text, size_t n, wchar_t w, int32_t x, long double d);
typedef struct opaque_thing *thing_t;
void use_thing(thing_t t);
void stop(int code) __attribute__((noreturn));
struct last { int a; long b; };
