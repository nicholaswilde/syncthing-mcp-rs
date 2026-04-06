#[cfg(test)]
mod tests {
    use crate::api::models::*;

    #[test]
    fn test_parse_folder_completion() {
        let json = r#"{
            "completion": 99.9937565835,
            "globalBytes": 156793013575,
            "needBytes": 9789241,
            "globalItems": 7823,
            "needItems": 412,
            "needDeletes": 0,
            "remoteState": "valid",
            "sequence": 12
        }"#;
        let completion: FolderCompletion = serde_json::from_str(json).unwrap();
        assert_eq!(completion.completion, 99.9937565835);
        assert_eq!(completion.global_bytes, 156793013575);
        assert_eq!(completion.need_bytes, 9789241);
        assert_eq!(completion.global_items, 7823);
        assert_eq!(completion.need_items, 412);
        assert_eq!(completion.need_deletes, 0);
        assert_eq!(completion.remote_state, "valid");
        assert_eq!(completion.sequence, 12);
    }
}
