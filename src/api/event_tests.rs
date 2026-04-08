#[cfg(test)]
mod tests {
    use crate::api::models::{Event, EventData};

    #[test]
    fn test_event_data_folder_state_changed() {
        let json = r#"{
            "id": 1,
            "type": "FolderStateChanged",
            "time": "2023-01-01T00:00:00Z",
            "data": {
                "folder": "f1",
                "from": "idle",
                "to": "syncing"
            }
        }"#;
        let event: Event =
            serde_json::from_str(json).expect("Should deserialize FolderStateChanged");
        match &event.data {
            Some(EventData::FolderStateChanged {
                folder,
                from,
                to,
                error,
            }) => {
                assert_eq!(folder, "f1");
                assert_eq!(from, "idle");
                assert_eq!(to, "syncing");
                assert!(error.is_none());
            }
            _ => panic!("Expected FolderStateChanged data"),
        }
        assert_eq!(
            event.summary(),
            "Folder 'f1' changed state from idle to syncing"
        );
    }

    #[test]
    fn test_event_data_device_connected() {
        let json = r#"{
            "id": 2,
            "type": "DeviceConnected",
            "time": "2023-01-01T00:00:00Z",
            "data": {
                "device": "d1",
                "addr": "1.2.3.4",
                "type": "tcp-client"
            }
        }"#;
        let event: Event = serde_json::from_str(json).expect("Should deserialize DeviceConnected");
        match &event.data {
            Some(EventData::DeviceConnected {
                device,
                addr,
                conn_type,
            }) => {
                assert_eq!(device, "d1");
                assert_eq!(addr, "1.2.3.4");
                assert_eq!(conn_type, "tcp-client");
            }
            _ => panic!("Expected DeviceConnected data"),
        }
        assert_eq!(
            event.summary(),
            "Device 'd1' connected via tcp-client at 1.2.3.4"
        );
    }

    #[test]
    fn test_event_data_device_disconnected() {
        let json = r#"{
            "id": 3,
            "type": "DeviceDisconnected",
            "time": "2023-01-01T00:00:00Z",
            "data": {
                "device": "d1",
                "error": "connection reset"
            }
        }"#;
        let event: Event =
            serde_json::from_str(json).expect("Should deserialize DeviceDisconnected");
        match &event.data {
            Some(EventData::DeviceDisconnected { device, error }) => {
                assert_eq!(device, "d1");
                assert_eq!(error, "connection reset");
            }
            _ => panic!("Expected DeviceDisconnected data"),
        }
        assert_eq!(
            event.summary(),
            "Device 'd1' disconnected: connection reset"
        );
    }

    #[test]
    fn test_event_data_local_index_updated() {
        let json = r#"{
            "id": 4,
            "type": "LocalIndexUpdated",
            "time": "2023-01-01T00:00:00Z",
            "data": {
                "folder": "f1",
                "filenames": ["a.txt", "b.txt"]
            }
        }"#;
        let event: Event =
            serde_json::from_str(json).expect("Should deserialize LocalIndexUpdated");
        match &event.data {
            Some(EventData::LocalIndexUpdated { folder, filenames }) => {
                assert_eq!(folder, "f1");
                assert_eq!(filenames, &vec!["a.txt".to_string(), "b.txt".to_string()]);
            }
            _ => panic!("Expected LocalIndexUpdated data"),
        }
        assert_eq!(
            event.summary(),
            "Local index updated for folder 'f1' (2 files)"
        );
    }

    #[test]
    fn test_event_data_generic() {
        let json = r#"{
            "id": 5,
            "type": "Starting",
            "time": "2023-01-01T00:00:00Z",
            "data": {
                "some": "value"
            }
        }"#;
        let event: Event = serde_json::from_str(json).expect("Should deserialize Generic event");
        match &event.data {
            Some(EventData::Generic(val)) => {
                assert_eq!(val["some"], "value");
            }
            _ => panic!("Expected Generic data"),
        }
        assert_eq!(event.summary(), "Event: Starting");
    }

    #[test]
    fn test_event_data_none() {
        let json = r#"{
            "id": 6,
            "type": "ListenAddressesChanged",
            "time": "2023-01-01T00:00:00Z",
            "data": null
        }"#;
        let event: Event =
            serde_json::from_str(json).expect("Should deserialize event with no data");
        assert!(event.data.is_none());
        assert_eq!(event.summary(), "Event: ListenAddressesChanged");
    }

    #[test]
    fn test_event_to_summary() {
        let json = r#"{
            "id": 1,
            "type": "FolderStateChanged",
            "time": "2023-01-01T00:00:00Z",
            "data": {
                "folder": "f1",
                "from": "idle",
                "to": "syncing"
            }
        }"#;
        let event: Event = serde_json::from_str(json).unwrap();
        let summary = event.to_summary();

        assert_eq!(summary.id, 1);
        assert_eq!(summary.event_type, "FolderStateChanged");
        assert_eq!(summary.time, "2023-01-01T00:00:00Z");
        assert_eq!(
            summary.summary,
            "Folder 'f1' changed state from idle to syncing"
        );
    }
}
