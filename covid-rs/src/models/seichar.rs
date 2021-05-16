use rand::Rng;

use crate::{
    epidemic::{EpiModel, Params, SEAIRLike, SEICHARLike, SEIRLike, SIRLike},
    sim::{State, StochasticUpdate},
};

use super::sir::SCR;

/// Enumeration used internally to distinguish Exposed, Infectious, Asymptomatic
/// Critical and Severe in SEICHAR.
#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub enum InnerSEICHAR<C> {
    Exposed(C),
    Infectious(C),
    Asymptomatic(C),
    Severe(C),
    Critical(C),
}

type SEICHAR<C> = SCR<InnerSEICHAR<C>, C>;

impl<C: State> State for SEICHAR<C> {}

impl<C: Clone> EpiModel for SEICHAR<C> {
    const CARDINALITY: usize = 7;
    const S: usize = 0;
    const D: usize = 7;

    type Disease = ();
    type Clinical = C;

    fn is_contagious(&self) -> bool {
        self.is_contaminated()
    }

    fn index(&self) -> usize {
        match self {
            Self::Susceptible => Self::S,
            Self::Contaminating(InnerSEICHAR::Exposed(_)) => Self::E,
            Self::Contaminating(InnerSEICHAR::Critical(_)) => Self::C,
            Self::Contaminating(InnerSEICHAR::Severe(_)) => Self::H,
            Self::Contaminating(InnerSEICHAR::Asymptomatic(_)) => Self::A,
            Self::Contaminating(InnerSEICHAR::Infectious(_)) => Self::I,
            Self::Recovered(_) => Self::R,
            Self::Dead(_) => Self::D,
        }
    }
}

impl<C: Clone> SIRLike for SEICHAR<C> {
    const I: usize = 2;
    const R: usize = 6;

    fn is_exposed(&self) -> bool {
        self.is_contaminating()
    }

    fn expose(&mut self, with: &Self::Clinical) {
        *self = self.epistate_from(&SCR::Contaminating(InnerSEICHAR::Exposed(with.clone())))
    }

    fn infect(&mut self, with: &Self::Clinical) {
        *self = self.epistate_from(&SCR::Contaminating(InnerSEICHAR::Infectious(with.clone())))
    }
}

impl<C: Clone> SEIRLike for SEICHAR<C> {
    const E: usize = 1;
}

impl<C: Clone> SEAIRLike for SEICHAR<C> {
    const A: usize = 5;
}

impl<C: Clone> SEICHARLike for SEICHAR<C> {
    const C: usize = 3;
    const H: usize = 4;
}

impl<C: State> StochasticUpdate<Params> for SEICHAR<C> {
    fn update_random<R: Rng>(&mut self, params: &Params, rng: &mut R) {
        let age = 40; // TODO
        self.update_inner_or(move |inner| {
            match inner {
                InnerSEICHAR::Exposed(c) => {
                    if rng.gen_bool(params.incubation_transition_prob()) {
                        if rng.gen_bool(params.prob_asymptomatic(age)) {
                            *inner = InnerSEICHAR::Asymptomatic(c.clone())
                        } else {
                            *inner = InnerSEICHAR::Infectious(c.clone())
                        }
                    }
                }
                InnerSEICHAR::Asymptomatic(c) => {
                    if rng.gen_bool(params.infectious_transition_prob()) {
                        return Some(SCR::Recovered(c.clone()));
                    }
                }
                InnerSEICHAR::Infectious(c) => {
                    if rng.gen_bool(params.infectious_transition_prob()) {
                        if rng.gen_bool(params.prob_severe(age)) {
                            *inner = InnerSEICHAR::Severe(c.clone())
                        } else {
                            return Some(SCR::Recovered(c.clone()));
                        }
                    }
                }
                InnerSEICHAR::Severe(c) => {
                    if rng.gen_bool(params.severe_transition_prob()) {
                        if rng.gen_bool(params.prob_critical(age)) {
                            *inner = InnerSEICHAR::Critical(c.clone())
                        } else {
                            return Some(SCR::Recovered(c.clone()));
                        }
                    }
                }
                InnerSEICHAR::Critical(c) => {
                    if rng.gen_bool(params.critical_transition_prob()) {
                        if rng.gen_bool(params.prob_death(age)) {
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
