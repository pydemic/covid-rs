use rand::Rng;

use crate::{
    epidemic::{EpiModel, Params, SEAIRLike, SEIRLike, SIRLike},
    sim::{State, StochasticUpdate},
};

use super::sir::SCR;

/// Enumeration used internally to distinguish Exposed, Infectious and Asymptomatic
/// in SEAIR.
#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub enum InnerSEAIR<C> {
    Exposed(C),
    Infectious(C),
    Asymptomatic(C),
}

type SEAIR<C> = SCR<InnerSEAIR<C>, C>;

impl<C: State> State for SEAIR<C> {}

impl<C: Clone> EpiModel for SEAIR<C> {
    const CARDINALITY: usize = 5;
    const S: usize = 0;
    const D: usize = 5;

    type Disease = ();
    type Clinical = C;

    fn index(&self) -> usize {
        match self {
            Self::Susceptible => Self::S,
            Self::Contaminating(InnerSEAIR::Exposed(_)) => Self::E,
            Self::Contaminating(InnerSEAIR::Asymptomatic(_)) => Self::A,
            Self::Contaminating(InnerSEAIR::Infectious(_)) => Self::I,
            Self::Recovered(_) => Self::R,
            Self::Dead(_) => Self::D,
        }
    }

    fn is_contagious(&self) -> bool {
        self.is_contaminating()
    }
}

impl<C: Clone> SIRLike for SEAIR<C> {
    const I: usize = 3;
    const R: usize = 4;

    fn is_exposed(&self) -> bool {
        self.index() == Self::E
    }

    fn expose(&mut self, with: &Self::Clinical) {
        *self = self.epistate_from(&SCR::Contaminating(InnerSEAIR::Exposed(with.clone())))
    }

    fn infect(&mut self, with: &Self::Clinical) {
        *self = self.epistate_from(&SCR::Contaminating(InnerSEAIR::Infectious(with.clone())))
    }
}

impl<C: Clone> SEIRLike for SEAIR<C> {
    const E: usize = 1;
}

impl<C: Clone> SEAIRLike for SEAIR<C> {
    const A: usize = 2;
}

impl<C: State> StochasticUpdate<Params> for SEAIR<C> {
    fn update_random<R: Rng>(&mut self, params: &Params, rng: &mut R) {
        // FIXME: implement age-independent parameters!
        let age = 40;

        self.update_inner_or(|inner| {
            match inner {
                InnerSEAIR::Exposed(c) => {
                    if rng.gen_bool(params.incubation_transition_prob()) {
                        if rng.gen_bool(params.prob_asymptomatic(40)) {
                            *inner = InnerSEAIR::Asymptomatic(c.clone())
                        } else {
                            *inner = InnerSEAIR::Infectious(c.clone())
                        }
                    }
                }
                InnerSEAIR::Asymptomatic(c) => {
                    if rng.gen_bool(params.infectious_transition_prob()) {
                        return Some(SCR::Recovered(c.clone()));
                    }
                }
                InnerSEAIR::Infectious(c) => {
                    if rng.gen_bool(params.infectious_transition_prob()) {
                        if rng.gen_bool(params.case_fatality_ratio(age)) {
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
