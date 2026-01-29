pub mod strength;

pub struct ForgettingConfig {
    pub min_age_days: u32,
    pub strength_threshold: f32,
}

impl Default for ForgettingConfig {
    fn default() -> Self {
        Self {
            min_age_days: 7,
            strength_threshold: 0.3,
        }
    }
}
