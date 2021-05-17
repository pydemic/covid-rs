use rand::Rng;

use crate::{
    epidemic::{EpiModel, Params, SEIRLike},
    prelude::Real,
    sim::{State, StochasticUpdate},
};

/// Enumeration used internally to distinguish Exposed from Infectious in SEIR.
#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub enum SEIR<C> {
    Susceptible,
    Exposed(C),
    Infectious(C),
    Recovered(C),
    Dead(C),
}

impl<C> SEIR<C> {
    pub fn clinical(&self) -> Option<C>
    where
        C: Clone,
    {
        match self {
            Self::Susceptible => None,
            Self::Exposed(c) | Self::Infectious(c) | Self::Recovered(c) | Self::Dead(c) => {
                Some(c.clone())
            }
        }
    }
}

impl<C: Default> Default for SEIR<C> {
    fn default() -> Self {
        Self::Susceptible
    }
}

impl<C: State> State for SEIR<C> {}

impl<C: Clone> EpiModel for SEIR<C> {
    const CARDINALITY: usize = 5;
    const S: usize = 0;
    const D: usize = 4;

    type Disease = ();
    type Clinical = C;

    fn index(&self) -> usize {
        match self {
            Self::Susceptible => Self::S,
            Self::Exposed(_) => Self::E,
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
        other.clinical().map(|c| *self = Self::Exposed(c)).is_some()
    }
}

impl<C: Clone> SEIRLike for SEIR<C> {
    const E: usize = 1;
    const I: usize = 2;
    const R: usize = 3;

    fn is_exposed(&self) -> bool {
        self.index() == Self::E
    }

    fn expose(&mut self, with: &Self::Clinical) {
        *self = Self::Exposed(with.clone())
    }

    fn infect(&mut self, with: &Self::Clinical) {
        *self = Self::Infectious(with.clone())
    }
}

impl<C: State> StochasticUpdate<Params> for SEIR<C> {
    fn update_random<R: Rng>(&mut self, params: &Params, rng: &mut R) {
        // FIXME: implement age-independent parameters!
        let age = 40;

        match self {
            Self::Exposed(c) => {
                if rng.gen_bool(params.incubation_transition_prob()) {
                    *self = Self::Infectious(c.clone())
                }
            }
            Self::Infectious(c) => {
                if rng.gen_bool(params.infectious_transition_prob()) {
                    if rng.gen_bool(params.infection_fatality_ratio(age)) {
                        *self = Self::Dead(c.clone());
                    } else {
                        *self = Self::Recovered(c.clone());
                    }
                }
            }
            _ => (),
        }
    }
}
