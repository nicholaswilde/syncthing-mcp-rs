use crate::tools::bandwidth::{BandwidthLimits, PerformanceProfile, ProfileSchedule};

#[cfg(test)]
mod tests {
    use super::*;

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
}
