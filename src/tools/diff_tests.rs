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
        println!("Diff: {}", diff);
        assert!(diff.contains("-line 2"));
        assert!(diff.contains("+line 2 mod"));
    }

    #[test]
    fn test_get_json_diff() {
        let original = r#"{"a": 1, "b": 2}"#;
        let conflict = r#"{"a": 1, "b": 3, "c": 4}"#;
        let diff = get_diff(original, conflict, DiffFormat::Json).unwrap();
        assert!(diff.contains("\"b\""));
        assert!(diff.contains("\"c\""));
    }

    #[test]
    fn test_get_yaml_diff() {
        let original = "a: 1\nb: 2";
        let conflict = "a: 1\nb: 3\nc: 4";
        let diff = get_diff(original, conflict, DiffFormat::Yaml).unwrap();
        assert!(diff.contains("b"));
        assert!(diff.contains("c"));
    }
}
