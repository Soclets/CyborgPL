/* For the oracle's tests: a struct with a field named like a Zag keyword, a function too, and an
   opaque struct (its definition is never given). */
struct buf { long size; int count; };
typedef struct handle handle_t;
long size(const struct buf *b);
handle_t *open_handle(const char *name);
void close_handle(handle_t *h);
