use crate::prelude::*;
use rand::distributions::WeightedIndex;
use rand::prelude::*;

/// Sample n ages from a non-empty vector of probabilities given as pairs
/// (age_group, prob). Probabilities do not need to be normalized and age
/// groups are iterpreted the range from the given number to the next value.
///
/// The last age group is assumed to have the same size as the penultimate one.
pub fn random_ages(n: usize, rng: &mut impl Rng, probs: AgeDistribution10) -> Vec<Age> {
    let distrib = WeightedIndex::new(&probs).unwrap();
    return (0..n)
        .map(|_| (10 * distrib.sample(rng) + rng.gen_range(0..10)) as Age)
        .collect();
}
