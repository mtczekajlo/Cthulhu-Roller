use crate::roller::{dice_rng::RealRng, roll::roll_die, success_level::SuccessLevel};

#[derive(Clone)]
pub struct ImproveResult {
    pub result: i32,
    pub success_level: SuccessLevel,
    pub threshold: i32,
}

impl ImproveResult {
    pub fn new(threshold: i32, result: i32) -> Self {
        let success_level = match result > threshold || result > 95 {
            true => SuccessLevel::Success,
            _ => SuccessLevel::Failure,
        };
        Self {
            threshold,
            result,
            success_level,
        }
    }
}

pub fn improve_skill(threshold: i32) -> ImproveResult {
    let mut rng = RealRng::new();
    ImproveResult::new(threshold, roll_die(&mut rng, 100))
}
