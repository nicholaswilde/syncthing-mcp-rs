use crate::tools::bandwidth::{BandwidthConfig, BandwidthLimits, PerformanceProfile, ProfileSchedule};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::bandwidth::ProfileManager;

    #[test]
    fn test_profile_logic() {
        let profile = PerformanceProfile {
            name: "working_hours".to_string(),
            limits: BandwidthLimits {
                max_recv_kbps: Some(100),
                max_send_kbps: Some(50),
            },
        };

        assert_eq!(profile.name, "working_hours");
        assert_eq!(profile.limits.max_recv_kbps, Some(100));
        assert_eq!(profile.limits.max_send_kbps, Some(50));
    }

    #[test]
    fn test_profile_manager_apply_profile() {
        let working_hours = PerformanceProfile {
            name: "working_hours".to_string(),
            limits: BandwidthLimits {
                max_recv_kbps: Some(100),
                max_send_kbps: Some(50),
            },
        };
        let full_speed = PerformanceProfile {
            name: "full_speed".to_string(),
            limits: BandwidthLimits {
                max_recv_kbps: Some(0),
                max_send_kbps: Some(0),
            },
        };

        let config = BandwidthConfig {
            profiles: vec![working_hours, full_speed],
            schedules: vec![],
            active_profile: None,
        };

        let mut manager = ProfileManager::new(config);
        
        // Apply working_hours
        let limits = manager.apply_profile("working_hours").unwrap();
        assert_eq!(limits.max_recv_kbps, Some(100));
        assert_eq!(manager.config.active_profile, Some("working_hours".to_string()));

        // Apply full_speed
        let limits = manager.apply_profile("full_speed").unwrap();
        assert_eq!(limits.max_recv_kbps, Some(0));
        assert_eq!(manager.config.active_profile, Some("full_speed".to_string()));

        // Apply non-existent
        assert!(manager.apply_profile("non_existent").is_none());
    }

    #[test]
    fn test_profile_manager_get_scheduled_profile() {
        let working_hours_profile = PerformanceProfile {
            name: "working_hours".to_string(),
            limits: BandwidthLimits {
                max_recv_kbps: Some(100),
                max_send_kbps: Some(50),
            },
        };
        let off_hours_profile = PerformanceProfile {
            name: "off_hours".to_string(),
            limits: BandwidthLimits {
                max_recv_kbps: Some(0),
                max_send_kbps: Some(0),
            },
        };

        let schedule = ProfileSchedule {
            profile_name: "working_hours".to_string(),
            days: vec!["Monday".to_string(), "Tuesday".to_string()],
            start_time: "09:00".to_string(),
            end_time: "17:00".to_string(),
        };

        let config = BandwidthConfig {
            profiles: vec![working_hours_profile, off_hours_profile],
            schedules: vec![schedule],
            active_profile: None,
        };

        let manager = ProfileManager::new(config);

        // Monday 10:00 -> working_hours
        let monday_10 = chrono::NaiveDate::from_ymd_opt(2026, 3, 23).unwrap() // Monday
            .and_hms_opt(10, 0, 0).unwrap();
        assert_eq!(manager.get_scheduled_profile_at(monday_10), Some("working_hours".to_string()));

        // Monday 08:00 -> None
        let monday_08 = chrono::NaiveDate::from_ymd_opt(2026, 3, 23).unwrap()
            .and_hms_opt(8, 0, 0).unwrap();
        assert_eq!(manager.get_scheduled_profile_at(monday_08), None);

        // Wednesday 10:00 -> None
        let wednesday_10 = chrono::NaiveDate::from_ymd_opt(2026, 3, 25).unwrap() // Wednesday
            .and_hms_opt(10, 0, 0).unwrap();
        assert_eq!(manager.get_scheduled_profile_at(wednesday_10), None);
    }
}
