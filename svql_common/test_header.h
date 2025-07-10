#ifdef __cplusplus
extern "C" {
#endif


#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef struct CSourceRange {
  const char *file;
  uintptr_t line_begin;
  uintptr_t col_begin;
  uintptr_t line_end;
  uintptr_t col_end;
} CSourceRange;

typedef struct CSourceLoc {
  const struct CSourceRange *ranges;
  uintptr_t len;
} CSourceLoc;

typedef struct CPattern {
  const char *file_loc;
  const char *const *in_ports;
  uintptr_t in_ports_len;
  const char *const *out_ports;
  uintptr_t out_ports_len;
  const char *const *inout_ports;
  uintptr_t inout_ports_len;
} CPattern;

/**
 * Free a string previously returned by any of the `*_to_string*` functions.
 */
void svql_free_string(char *s);

/**
 * Parse a single SourceRange from a C string.
 * Returns null on bad input.
 * Caller owns the returned CSourceRange* and must call `svql_source_range_free`.
 */
struct CSourceRange *svql_source_range_parse(const char *s);

/**
 * Turn a CSourceRange into its compact string form.
 * Returns a malloc’d C string (must be freed by `svql_free_string`).
 */
char *svql_source_range_to_string(const struct CSourceRange *r);

/**
 * Turn a CSourceRange into its “pretty” multi‐line form.
 * Returns a malloc’d C string (must be freed by `svql_free_string`).
 */
char *svql_source_range_to_string_pretty(const struct CSourceRange *r);

/**
 * Free a CSourceRange previously returned by `svql_source_range_parse`.
 */
void svql_source_range_free(struct CSourceRange *ptr);

/**
 * Parse a SourceLoc (a list of ranges) from a C‐string, using separator `sep`.
 * Returns null on error. Caller must free with `svql_source_loc_free`.
 */
struct CSourceLoc *svql_source_loc_parse(const char *s, char sep);

/**
 * Turn a CSourceLoc into its compact string form (using `sep`).
 * Returns malloc’d C string, free with `svql_free_string`.
 */
char *svql_source_loc_to_string(const struct CSourceLoc *loc, char sep);

/**
 * Turn a CSourceLoc into its “pretty” multi‐line form.
 * Returns malloc’d C string, free with `svql_free_string`.
 */
char *svql_source_loc_to_string_pretty(const struct CSourceLoc *loc);

/**
 * Free a CSourceLoc and all of its inner allocations.
 */
void svql_source_loc_free(struct CSourceLoc *ptr);

/**
 * Is this SourceLoc empty?
 */
bool svql_source_loc_empty(const struct CSourceLoc *loc);

/**
 * Append one CSourceRange to a CSourceLoc (deep‐copies all strings).
 */
void svql_source_loc_append(struct CSourceLoc *loc, const struct CSourceRange *range);

struct CPattern *cpattern_new(const char *file_loc,
                              const char *const *in_ports,
                              uintptr_t in_ports_len,
                              const char *const *out_ports,
                              uintptr_t out_ports_len,
                              const char *const *inout_ports,
                              uintptr_t inout_ports_len);

void cpattern_free(struct CPattern *ptr);

#ifdef __cplusplus
}
#endif
