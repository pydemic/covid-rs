use rand::Rng;

use crate::{epidemic::{EpiModel, Params, SEICHARLike, SEIRLike}, prelude::Real, sim::{State, StochasticUpdate}};

/// Enumeration used internally to distinguish Exposed, Infectious and Asymptomatic
/// in SEAIR.
#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub enum SEAIR<C> {
    Susceptible,
    Exposed(C),
    Infectious(C),
    Asymptomatic(C),
    Recovered(C),
    Dead(C),
}

impl<C> SEAIR<C> {
    pub fn clinical(&self) -> Option<C>
    where
        C: Clone,
    {
        match self {
            Self::Susceptible => None,
            Self::Exposed(c)
            | Self::Asymptomatic(c)
            | Self::Infectious(c)
            | Self::Recovered(c)
            | Self::Dead(c) => Some(c.clone()),
        }
    }
}

impl<C> Default for SEAIR<C> {
    fn default() -> Self {
        return Self::Susceptible;
    }
}

impl<C: State> State for SEAIR<C> {}

impl<C: Clone> EpiModel for SEAIR<C> {
    const CARDINALITY: usize = 6;
    const S: usize = 0;
    const D: usize = 5;

    type Disease = ();
    type Clinical = C;

    fn index(&self) -> usize {
        match self {
            Self::Susceptible => Self::S,
            Self::Exposed(_) => Self::E,
            Self::Asymptomatic(_) => Self::A,
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
            Self::Asymptomatic(_) => 0.42,
            _ => 0.0,
        }
    }

    fn transfer_contamination_from(&mut self, other: &Self) -> bool {
        other.clinical().map(|c| *self = Self::Exposed(c)).is_some()
    }
}

impl<C: Clone> SEIRLike for SEAIR<C> {
    const E: usize = 1;
    const I: usize = 3;
    const R: usize = 4;

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

impl<C: Clone> SEICHARLike for SEAIR<C> {
    const C: usize = Self::I;
    const H: usize = Self::I;
    const A: usize = 2;
}

impl<C: State> StochasticUpdate<Params> for SEAIR<C> {
    fn update_random<R: Rng>(&mut self, params: &Params, rng: &mut R) {
        // FIXME: implement age-independent parameters!
        let age = 40;

        match self {
            Self::Exposed(c) => {
                if rng.gen_bool(params.incubation_transition_prob()) {
                    if rng.gen_bool(params.prob_asymptomatic(40)) {
                        *self = Self::Asymptomatic(c.clone())
                    } else {
                        *self = Self::Infectious(c.clone())
                    }
                }
            }
            Self::Asymptomatic(c) => {
                if rng.gen_bool(params.infectious_transition_prob()) {
                    *self = Self::Recovered(c.clone());
                }
            }
            Self::Infectious(c) => {
                if rng.gen_bool(params.infectious_transition_prob()) {
                    if rng.gen_bool(params.case_fatality_ratio(age)) {
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
