use rand::Rng;

use crate::{
    epidemic::{EpiModel, Params, SEIRLike, SIRLike},
    sim::{State, StochasticUpdate},
};

use super::sir::SCR;

/// Enumeration used internally to distinguish Exposed from Infectious in SEIR.
#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub(crate) enum InnerSEIR<C> {
    Exposed(C),
    Infectious(C),
}

impl<C: Default> Default for InnerSEIR<C> {
    fn default() -> Self {
        Self::Exposed(C::default())
    }
}

impl<C: State> State for InnerSEIR<C> {}

type SEIR<C> = SCR<InnerSEIR<C>, C>;

impl<C: Clone> EpiModel for SEIR<C> {
    const CARDINALITY: usize = 5;
    const S: usize = 0;
    const D: usize = 4;

    type Disease = ();
    type Clinical = C;

    fn index(&self) -> usize {
        match self {
            Self::Susceptible => Self::S,
            Self::Contaminating(InnerSEIR::Exposed(_)) => Self::E,
            Self::Contaminating(InnerSEIR::Infectious(_)) => Self::I,
            Self::Recovered(_) => Self::R,
            Self::Dead(_) => Self::D,
        }
    }

    fn is_contagious(&self) -> bool {
        self.is_contaminated()
    }
}

impl<C: Clone> SIRLike for SEIR<C> {
    const I: usize = 2;
    const R: usize = 3;

    fn is_exposed(&self) -> bool {
        self.index() == Self::E
    }

    fn expose(&mut self, with: &Self::Clinical) {
        expose_seir(self, with)
    }

    fn infect(&mut self, with: &Self::Clinical) {
        infect_seir(self, with)
    }
}

pub(crate) fn expose_seir<C: Clone>(seir: &mut SEIR<C>, with: &C) {
    *seir = seir.epistate_from(&SCR::Contaminating(InnerSEIR::Exposed(with.clone())))
}

pub(crate) fn infect_seir<C: Clone>(seir: &mut SEIR<C>, with: &C) {
    *seir = seir.epistate_from(&SCR::Contaminating(InnerSEIR::Infectious(with.clone())))
}

impl<C: Clone> SEIRLike for SEIR<C> {
    const E: usize = 1;
}

impl<C: State> StochasticUpdate<Params> for SEIR<C> {
    fn update_random<R: Rng>(&mut self, params: &Params, rng: &mut R) {
        // FIXME: implement age-independent parameters!
        let age = 40;

        self.update_inner_or(|inner| {
            match inner {
                InnerSEIR::Exposed(c) => {
                    if rng.gen_bool(params.incubation_transition_prob()) {
                        *inner = InnerSEIR::Infectious(c.clone())
                    }
                }
                InnerSEIR::Infectious(c) => {
                    if rng.gen_bool(params.infectious_transition_prob()) {
                        if rng.gen_bool(params.infection_fatality_ratio(age)) {
                            return Some(SCR::Dead(c.clone()));
                        } else {
                            return Some(SCR::Recovered(c.clone()));
                        }
                    }
                }
            }
            return None;
        })
    }
}
