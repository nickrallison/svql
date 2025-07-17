#ifndef SVQL_COMMON_H
#define SVQL_COMMON_H

#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>

/// C FFI equivalent of CellData
struct CCellData {
  char *cell_name;
  uintptr_t cell_index;
};

/// C FFI representation of a string-to-string map entry
struct CStringMapEntry {
  char *key;
  char *value;
};

/// C FFI representation of a string-to-string map
struct CStringMap {
  CStringMapEntry *entries;
  uintptr_t len;
};

/// C FFI representation of a CellData-to-CellData map entry
struct CCellDataMapEntry {
  CCellData key;
  CCellData value;
};

/// C FFI representation of a CellData-to-CellData map
struct CCellDataMap {
  CCellDataMapEntry *entries;
  uintptr_t len;
};

struct CMatch {
  CStringMap *port_map;
  CCellDataMap *cell_map;
};

/// C FFI representation of MatchList
struct CMatchList {
  CMatch **matches;
  uintptr_t len;
};

struct CPattern {
  const char *file_loc;
  const char *const *in_ports;
  uintptr_t in_ports_len;
  const char *const *out_ports;
  uintptr_t out_ports_len;
  const char *const *inout_ports;
  uintptr_t inout_ports_len;
};

struct CSourceRange {
  const char *file;
  uintptr_t line_begin;
  uintptr_t col_begin;
  uintptr_t line_end;
  uintptr_t col_end;
};

struct CSourceLoc {
  const CSourceRange *ranges;
  uintptr_t len;
};

struct CStringPair {
  char *first;
  char *second;
};

struct CStringVec {
  char *key;
  char **values_ptr;
  uintptr_t values_len;
};

struct CStringVecPair {
  char *key;
  char **first_values_ptr;
  uintptr_t first_values_len;
  char **second_values_ptr;
  uintptr_t second_values_len;
};

struct CConfig {
  bool verbose;
  bool const_ports;
  bool nodefaultswaps;
  CStringPair *compat_pairs_ptr;
  uintptr_t compat_pairs_len;
  CStringVec *swap_ports_ptr;
  uintptr_t swap_ports_len;
  CStringVecPair *perm_ports_ptr;
  uintptr_t perm_ports_len;
  char **cell_attr_ptr;
  uintptr_t cell_attr_len;
  char **wire_attr_ptr;
  uintptr_t wire_attr_len;
  bool ignore_parameters;
  CStringPair *ignore_param_ptr;
  uintptr_t ignore_param_len;
};

extern "C" {

/// Create a new CCellData
CCellData *ccelldata_new(const char *cell_name, uintptr_t cell_index);

/// Serialize CCellData to JSON C string
char *ccelldata_serialize(const CCellData *ccell_data);

/// Free CCellData memory
void ccelldata_free(CCellData *ccell_data);

/// Create a new CMatch
CMatch *cmatch_new();

/// Add a port to CMatch
void cmatch_add_port(CMatch *cmatch, const char *key, const char *value);

/// Add a celldata to CMatch
void cmatch_add_celldata(CMatch *cmatch, const CCellData *key, const CCellData *value);

/// Serialize CMatch to JSON C string
char *cmatch_serialize(const CMatch *cmatch);

/// Free CMatch memory
void cmatch_free(CMatch *cmatch);

/// Free a JSON C string returned by serialize functions
void free_json_string(char *json_str);

/// Create a new CMatchList
CMatchList *cmatchlist_new();

/// Add a CMatch to a CMatchList
void cmatchlist_add_match(CMatchList *cmatch_list, CMatch *cmatch);

/// Serialize CMatchList to JSON C string
char *cmatchlist_serialize(const CMatchList *cmatch_list);

/// Free CMatchList memory
void cmatchlist_free(CMatchList *cmatch_list);

CPattern *cpattern_new(const char *file_loc,
                       const char *const *in_ports,
                       uintptr_t in_ports_len,
                       const char *const *out_ports,
                       uintptr_t out_ports_len,
                       const char *const *inout_ports,
                       uintptr_t inout_ports_len);

void cpattern_free(CPattern *ptr);

char *cpattern_to_json(const CPattern *pattern);

CPattern *cpattern_from_json(const char *json_str);

void cpattern_json_free(char *json_str);

/// Free a string previously returned by any of the `*_to_string*` functions.
void svql_free_string(char *s);

/// Parse a single SourceRange from a C string.
/// Returns null on bad input.
/// Caller owns the returned CSourceRange* and must call `svql_source_range_free`.
CSourceRange *svql_source_range_parse(const char *s);

/// Turn a CSourceRange into its compact string form.
/// Returns a malloc’d C string (must be freed by `svql_free_string`).
char *svql_source_range_to_string(const CSourceRange *r);

/// Turn a CSourceRange into its “pretty” multi‐line form.
/// Returns a malloc’d C string (must be freed by `svql_free_string`).
char *svql_source_range_to_string_pretty(const CSourceRange *r);

/// Free a CSourceRange previously returned by `svql_source_range_parse`.
void svql_source_range_free(CSourceRange *ptr);

/// Parse a SourceLoc (a list of ranges) from a C‐string, using separator `sep`.
/// Returns null on error. Caller must free with `svql_source_loc_free`.
CSourceLoc *svql_source_loc_parse(const char *s, char sep);

/// Turn a CSourceLoc into its compact string form (using `sep`).
/// Returns malloc’d C string, free with `svql_free_string`.
char *svql_source_loc_to_string(const CSourceLoc *loc, char sep);

/// Turn a CSourceLoc into its “pretty” multi‐line form.
/// Returns malloc’d C string, free with `svql_free_string`.
char *svql_source_loc_to_string_pretty(const CSourceLoc *loc);

char *svql_source_loc_to_json(CSourceLoc *s);

/// Free a CSourceLoc and all of its inner allocations.
void svql_source_loc_free(CSourceLoc *ptr);

/// Is this SourceLoc empty?
bool svql_source_loc_empty(const CSourceLoc *loc);

/// Append one CSourceRange to a CSourceLoc (deep‐copies all strings).
void svql_source_loc_append(CSourceLoc *loc, const CSourceRange *range);

char *config_to_json(const CConfig *config);

CConfig *config_from_json(const char *json_str);

void free_config(CConfig *config);

}  // extern "C"

#endif  // SVQL_COMMON_H

/* Text to put at the end of the generated file */
