pub fn split_inclusive<'a>(input: &'a str, pattern: &[char]) -> Vec<&'a str> {
    let mut slices = Vec::new();
    let mut start = 0;
    for (end, _) in input.match_indices(pattern) {
        if start != end {
            slices.push(&input[start..end]);
        }
        start = end;
    }
    slices.push(&input[start..]);
    slices
}

#[cfg(feature = "character-sheet")]
pub fn to_uppercase_first_letter(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}
