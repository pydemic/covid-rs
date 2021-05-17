use rand::Rng;

use crate::{epidemic::{EpiModel, Params, SIRLike}, prelude::Real, sim::{State, StochasticUpdate}};

/// Concrete implementation of the SIR model. This model is generic over a
/// clinical parameter type C. If no distinction should be made between different
/// clinical states besides being in ant of the Susceptible, Infectious, Recovered
/// states, C can be safely set to ().
#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub enum SIR<C> {
    Susceptible,
    Infectious(C),
    Recovered(C),
    Dead(C),
}

impl<C> SIR<C> {
    pub fn clinical(&self) -> Option<C>
    where
        C: Clone,
    {
        match self {
            Self::Susceptible => None,
            Self::Infectious(c) | Self::Recovered(c) | Self::Dead(c) => Some(c.clone()),
        }
    }
}

impl<C: State> State for SIR<C> {}

impl<C> Default for SIR<C> {
    fn default() -> Self {
        Self::Susceptible
    }
}

impl<C: Clone> EpiModel for SIR<C> {
    const CARDINALITY: usize = 4;
    const S: usize = 0;
    const D: usize = 3;
    type Disease = ();
    type Clinical = C;

    fn index(&self) -> usize {
        match self {
            Self::Susceptible => Self::S,
            Self::Infectious(_) => Self::I,
            Self::Recovered(_) => Self::R,
            Self::Dead(_) => Self::D,
        }
    }

    fn new_infectious_with(clinical: &Self::Clinical) -> Self {
        Self::Infectious(clinical.clone())
    }

    fn contagion_odds(&self) -> Real {
        match self {
            Self::Infectious(_) => 1.0,
            _ => 0.0,
        }
    }

    fn transfer_contamination_from(&mut self, other: &Self) -> bool {
        other
            .clinical()
            .map(|c| *self = Self::Infectious(c))
            .is_some()
    }
}

impl<C: Clone> SIRLike for SIR<C> {
    const I: usize = 1;
    const R: usize = 2;

    fn is_exposed(&self) -> bool {
        self.is_infectious()
    }

    fn infect(&mut self, with: &Self::Clinical) {
        *self = Self::Infectious(with.clone())
    }
}

impl<C: State> StochasticUpdate<Params> for SIR<C> {
    fn update_random<R: Rng>(&mut self, params: &Params, rng: &mut R) {
        match self {
            Self::Infectious(c) => {
                if rng.gen_bool(params.infectious_transition_prob()) {
                    *self = Self::Recovered(c.clone())
                }
            }
            _ => (),
        }
    }
}
