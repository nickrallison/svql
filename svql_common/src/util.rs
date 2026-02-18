/// Sanitizes a Rust type name by replacing special characters.
///
/// Converts `::` to `_`, removes angle brackets and commas, normalizes underscores.
/// This is useful for generating safe identifiers from type names.
pub fn sanitize_type_name(type_name: &str) -> String {
    type_name
        .replace("::", "_")
        .replace(['<', '>', ','], "_")
        .replace(' ', "")
        .replace("__", "_")
        .trim_matches('_')
        .to_string()
}

/// Returns the short name of a type by extracting the last segment after `::`.
///
/// For example, `std::collections::HashMap` becomes `HashMap`.
pub fn short_type_name(type_name: &str) -> &str {
    type_name.rsplit("::").next().unwrap_or(type_name)
}
