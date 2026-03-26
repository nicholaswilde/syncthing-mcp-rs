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
}
