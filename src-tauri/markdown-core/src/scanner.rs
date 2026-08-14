/// Find the first occurrence of `target` starting at `start`.
pub fn find_char(chars: &[char], start: usize, target: char) -> Option<usize> {
    for i in start..chars.len() {
        if chars[i] == target {
            return Some(i);
        }
    }
    None
}

/// Find the first occurrence of a doubled `target` (e.g. `**`, `~~`).
pub fn find_double(chars: &[char], start: usize, target: char) -> Option<usize> {
    if start >= chars.len() {
        return None;
    }
    for i in start..chars.len() - 1 {
        if chars[i] == target && chars[i + 1] == target {
            return Some(i);
        }
    }
    None
}
