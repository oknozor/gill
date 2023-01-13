// This filter does not have extra arguments
pub fn sha_digest<T: std::fmt::Display>(s: T) -> askama::Result<String> {
    let s = s.to_string();
    Ok(s[0..7].to_string())
}
