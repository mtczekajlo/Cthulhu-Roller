pub mod attribute_roll;
pub mod battle;
pub mod croll;
pub mod dice_rng;
pub mod improve_roll;
pub mod modifier_dice;
pub mod roll;
pub mod success_level;

#[cfg(test)]
mod tests {
    use crate::roller::dice_rng::DiceRng;
    use crate::roller::roll::RollRegex;
    use mockall::{predicate::*, *};
    use rstest::rstest;

    #[automock]
    trait MockableDiceRng {
        fn random_range(&mut self, range: std::ops::RangeInclusive<i32>) -> i32;
    }

    impl DiceRng for MockMockableDiceRng {
        fn random_range(&mut self, range: std::ops::RangeInclusive<i32>) -> i32 {
            MockableDiceRng::random_range(self, range)
        }
    }

    #[rstest]
    #[case("2", vec![RollRegex::new(0,0,1.0,2)])]
    #[case("+2", vec![RollRegex::new(0,0,1.0,2)])]
    #[case("-2", vec![RollRegex::new(0,0,1.0,-2)])]
    #[case("k10", vec![RollRegex::new(1,10,1.0,0)])]
    #[case("1k10", vec![RollRegex::new(1,10,1.0,0)])]
    #[case("k10x3", vec![RollRegex::new(1,10,3.0,0)])]
    #[case("k10x0.5", vec![RollRegex::new(1,10,0.5,0)])]
    #[case("k10+2", vec![RollRegex::new(1,10,1.0,0),RollRegex::new(0,0,1.0,2)])]
    #[case("k10x3+2", vec![RollRegex::new(1,10,3.0,0),RollRegex::new(0,0,1.0,2)])]
    #[case("k10+k10", vec![RollRegex::new(1,10,1.0,0),RollRegex::new(1,10,1.0,0)])]
    #[case("k10-k10", vec![RollRegex::new(1,10,1.0,0),RollRegex::new(1,10,-1.0,0)])]
    #[case("1k10+1k10", vec![RollRegex::new(1,10,1.0,0),RollRegex::new(1,10,1.0,0)])]
    #[case("1k10-1k10", vec![RollRegex::new(1,10,1.0,0),RollRegex::new(1,10,-1.0,0)])]
    #[case("2k10+2k10", vec![RollRegex::new(2,10,1.0,0),RollRegex::new(2,10,1.0,0)])]
    #[case("2k10-2k10", vec![RollRegex::new(2,10,1.0,0),RollRegex::new(2,10,-1.0,0)])]
    fn test_roll_parse(#[case] query: &str, #[case] expected: Vec<RollRegex>) {
        use crate::roller::roll::roll_parse;

        let rr = roll_parse(query);
        let rr = rr.unwrap();
        dbg!(&rr);
        assert_eq!(rr, expected);
    }

    #[rstest]
    #[case("2", 2)]
    #[case("+2", 2)]
    #[case("-2", 0)]
    #[case("k10", 5)]
    #[case("1k10", 5)]
    #[case("k10+2", 5+2)]
    #[case("k10x3", 5*3)]
    #[case("k10x0.5", 3)]
    #[case("k10x3-2", 5*3-2)]
    #[case("k10-2", 5-2)]
    #[case("k10x3-2", 5*3-2)]
    #[case("1k10", 5)]
    #[case("1k10+2", 5+2)]
    #[case("1k10x3+2", 5*3+2)]
    #[case("1k10-2", 5-2)]
    #[case("1k10x3-2", 5*3-2)]
    #[case("k10+k10", 5+5)]
    #[case("1k10+k10", 5+5)]
    #[case("k10+1k10", 5+5)]
    #[case("1k10+1k10", 5+5)]
    #[case("k10+2k10", 5+2*5)]
    #[case("1k10+2k10", 5+2*5)]
    #[case("k10+2k10", 5+2*5)]
    #[case("1k10+2k10", 5+2*5)]
    #[case("k10+2+k10+2", 5+2+5+2)]
    #[case("k10x3+2+k10x3+2", 5*3+2+5*3+2)]
    #[case("k10-2+k10-2", 5-2+5-2)]
    #[case("k10x3-2+k10x3-2", 5*3-2+5*3-2)]
    #[case("k10-k10", 0)]
    #[case("k10+2-k10+2", 5+2-5+2)]
    #[case("k10x3+2-k10x3+2", 5*3+2-5*3+2)]
    #[case("k10-2-k10-2", 0)]
    #[case("k10x3-2-k10x3-2", 0)]
    fn test_roll_impl(#[case] query: &str, #[case] expected: i32) {
        use crate::roller::roll::roll_impl;

        let mut mr = MockMockableDiceRng::new();
        mr.expect_random_range().returning(|_| 5);
        let dr = roll_impl(&mut mr, query);
        let dr = dr.unwrap();
        dbg!(&dr);
        assert_eq!(dr.result(), expected);
    }

    #[cfg(feature = "character-sheet")]
    #[rstest]
    #[case("2", 2)]
    #[case("+2", 2)]
    #[case("-2", 0)]
    #[case("k4", 4)]
    #[case("k10", 10)]
    #[case("k10+k6", 16)]
    #[case("k3+k4", 7)]
    #[case("1k4", 4)]
    #[case("1k10", 10)]
    #[case("1k10+1k6", 16)]
    #[case("1k3+1k4", 7)]
    #[case("k10-k6", 4)]
    #[case("k3-k4", 0)]
    #[case("1k10-1k6", 4)]
    #[case("1k3-1k4", 0)]
    #[case("1k3+2", 5)]
    #[case("1k3-2", 1)]
    fn test_roll_max_result(#[case] query: &str, #[case] expected: i32) {
        use crate::roller::roll::get_roll_max;

        let rmr = get_roll_max(query);
        let rmr = rmr.unwrap();
        dbg!(&rmr);
        assert_eq!(rmr.result(), expected);
    }
}
