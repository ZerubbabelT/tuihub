pub fn truncate_with_ellipsis(input: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }

    let chars: Vec<char> = input.chars().collect();
    if chars.len() <= max_chars {
        return input.to_string();
    }
    if max_chars == 1 {
        return ".".to_string();
    }

    let mut out = chars[..max_chars - 1].iter().collect::<String>();
    out.push('…');
    out
}
