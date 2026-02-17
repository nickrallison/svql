pub fn sanitize_type_name(type_name: &str) -> String {
    type_name
        .replace("::", "_")
        .replace(['<', '>', ','], "_")
        .replace(' ', "")
        .replace("__", "_")
        .trim_matches('_')
        .to_string()
}

pub fn short_type_name(type_name: &str) -> &str {
    type_name.rsplit("::").next().unwrap_or(type_name)
}
