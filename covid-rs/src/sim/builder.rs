use rand::Rng;

use super::{HasAge, Population, State};
use crate::{
    prelude::{Age, AgeCount10, AgeDistribution10},
    utils::random_ages,
};
use std::iter;

/// Creates a new population of n individuals starting with the default state.
pub fn new_population<P>(n: usize) -> P
where
    P: Population,
    P::State: State + Default,
{
    Population::from_states(iter::repeat(P::State::default()).take(n))
}

/// Creates a new population with ages.
pub fn new_population_from_ages<P, R>(counts: AgeCount10, rng: &mut R) -> P
where
    P: Population,
    P::State: State + Default + HasAge,
    R: Rng,
{
    let mut data: Vec<P::State> = vec![];
    for (i, &n) in counts.iter().enumerate() {
        for _ in 0..n {
            let mut st: P::State = Default::default();
            let start = i * 10;
            st.set_age(rng.gen_range(start..start + 10) as Age);
            data.push(st);
        }
    }
    return Population::from_states(data);
}

/// Creates a new population with ages.
pub fn new_population_from_distribution<P, R>(n: usize, distrib: AgeDistribution10, rng: &mut R) -> P
where
    P: Population,
    P::State: State + Default + HasAge,
    R: Rng,
{
    let ages = random_ages(n, rng, distrib);
    let mut pop: P = new_population(n);
    pop.each_agent_mut(|i, st: &mut P::State| {
        st.set_age(ages[i]);
    });
    return pop;
}
