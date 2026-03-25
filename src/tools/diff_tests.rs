#[cfg(test)]
mod tests {
    use crate::tools::diff::{get_text_diff, DiffFormat, get_diff};

    #[test]
    fn test_get_text_diff() {
        let original = "line 1\nline 2\nline 3";
        let conflict = "line 1\nline 2 modified\nline 3\nline 4";
        let diff = get_text_diff(original, conflict);
        assert!(diff.contains("-line 2"));
        assert!(diff.contains("+line 2 modified"));
        assert!(diff.contains("+line 4"));
    }

    #[test]
    fn test_get_diff_auto_text() {
        let original = "line 1\nline 2";
        let conflict = "line 1\nline 2 mod";
        let diff = get_diff(original, conflict, DiffFormat::Auto).unwrap();
        assert!(diff.contains("-line 2"));
        assert!(diff.contains("+line 2 mod"));
    }
}
