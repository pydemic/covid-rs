use rand::Rng;

use crate::{
    epidemic::{EpiModel, Params, SIRLike},
    sim::{State, StochasticUpdate},
};

/// Base enumeration that describes all SIR-like models. We group all states
/// that may contaminate into a single Contaminating() enumeration, which can
/// be subdivided later into different categories such as Exposed, Infectious,
/// Asymptomatic, etc.
#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub enum SCR<T, C> {
    Susceptible,
    Contaminating(T),
    Recovered(C),
    Dead(C),
}

impl<T, C> SCR<T, C> {
    pub(crate) fn is_contaminating(&self) -> bool {
        match self {
            Self::Contaminating(_) => true,
            _ => false,
        }
    }

    pub(crate) fn update_inner(&mut self, f: impl FnOnce(&mut T))
    where
        C: Clone,
    {
        match self {
            Self::Contaminating(inner) => f(inner),
            _ => (),
        }
    }

    pub(crate) fn update_inner_or(&mut self, f: impl FnOnce(&mut T) -> Option<Self>)
    where
        C: Clone,
    {
        match self {
            Self::Contaminating(inner) => {
                if let Some(st) = f(inner) {
                    *self = st;
                }
            }
            _ => ()
        }
    }
}

/// Concrete implementation of the SIR model. This model is generic over a
/// clinical parameter type C. If no distinction should be made between different
/// clinical states besides being in ant of the Susceptible, Infectious, Recovered
/// states, C can be safely set to ().
pub type SIR<C> = SCR<C, C>;

impl<T: State, C: State> State for SCR<T, C> {}

impl<T, C> Default for SCR<T, C> {
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
            Self::Contaminating(_) => Self::I,
            Self::Recovered(_) => Self::R,
            Self::Dead(_) => Self::D,
        }
    }

    fn is_contagious(&self) -> bool {
        self.is_contaminating()
    }
}

impl<C: Clone> SIRLike for SIR<C> {
    const I: usize = 1;
    const R: usize = 2;

    fn is_exposed(&self) -> bool {
        self.is_infectious()
    }

    fn infect(&mut self, with: &Self::Clinical) {
        *self = self.epistate_from(&Self::Contaminating(with.clone()))
    }
}

impl<C: State> StochasticUpdate<Params> for SIR<C> {
    fn update_random<R: Rng>(&mut self, params: &Params, rng: &mut R) {
        match self {
            Self::Contaminating(c) => {
                if rng.gen_bool(params.infectious_transition_prob()) {
                    *self = Self::Recovered(c.clone())
                }
            }
            _ => (),
        }
    }
}
