#![warn(clippy::pedantic)]
#![allow(clippy::redundant_field_names, clippy::if_not_else,
         clippy::cast_possible_truncation, clippy::cast_precision_loss, clippy::cast_sign_loss,
         clippy::wildcard_imports)]

mod common;
mod parse_data;
mod calculate_extrema;
mod calculate_pos_distribution;

pub use common::{ZombieType, PosDistribution};
pub use calculate_extrema::calculate_extrema;
pub use calculate_pos_distribution::calculate_pos_distribution;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extrema() {
        let result = calculate_extrema(ZombieType::GigaGargantuar, &[], 225);
        assert!((result.0 - 788.594).abs() < 1e-3, "{:?}", result);
        assert!((result.1 - 817.877).abs() < 1e-3, "{:?}", result);

        let result = calculate_extrema(ZombieType::Gargantuar, &[100], 949);
        assert!((result.0 - 781.973).abs() < 1e-3, "{:?}", result);
        assert!((result.1 - 817.944).abs() < 1e-3, "{:?}", result);

        let result = calculate_extrema(ZombieType::Regular, &[200], 2500);
        assert!((result.0 - 551.467).abs() < 1e-3, "{:?}", result);
        assert!((result.1 - 687.030).abs() < 1e-3, "{:?}", result);
    }
}
