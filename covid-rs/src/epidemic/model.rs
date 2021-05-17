use crate::{prelude::Real, sim::State};
use paste::paste;
use rand::Rng;
use serde::{Deserialize, Serialize};

/// Basic trait for all compartment-like epidemic models. This includes all the
/// SIR family of models and possibly other more generic cases.
///
/// An Epi model has a defined CARDINALITY that designates the number of
/// different basic states an agent can be in. Those states can have further
/// attached information (e.g., is infected with some given severity level) which
/// are not accounted for the CARDINALITY of a model.
///
/// The index() method converts a state to the corresponding integer representation
pub trait EpiModel: Sized + Clone {
    /// Maximum number of epidemiological states
    const CARDINALITY: usize;

    // Index of the susceptible state.
    const S: usize;

    // Index of the dead state.
    const D: usize;

    /// A type that represents the disease associated with the epidemiological
    /// model. Use the unity type to signal an abstract epidemiological models
    /// not associated with a concrete disease.
    type Disease;

    /// An additional clinical state that may describe other parameters of the
    /// contamination like severity, access to healthcare, etc.
    type Clinical;

    /// The index associated with the current epidemiological state. This is a
    /// simple mapping that usually depends on the epidemic model. For a SIR-like
    /// model, for instance, can be something like susceptible => 0,
    /// infectious => 1, recovered => 2, dead => 3.
    fn index(&self) -> usize;

    /// Create new infectious individual with the given clinical parameters.
    fn new_infectious_with(clinical: &Self::Clinical) -> Self;

    fn new_infectious() -> Self
    where
        Self::Clinical: Default,
    {
        Self::new_infectious_with(&Self::Clinical::default())
    }

    /// Return true if agent has been contaminated with disease irrespective of
    /// the current clinical state.
    ///
    /// The default implementation simply negates self.is_susceptible().
    fn is_contaminated(&self) -> bool {
        !self.is_susceptible()
    }

    /// Return true if agent is susceptible to be contaminated with disease
    /// irrespective of the current clinical state.
    fn is_susceptible(&self) -> bool {
        self.index() == Self::S
    }

    /// Return true if agent is able to contaminate other agents. It must return
    /// true even if the probability of contamination is very low.
    fn is_contagious(&self) -> bool {
        self.contagion_odds() > 0.0
    }

    /// Return the relative probability of contamination from self. Usually this
    /// should be equal to 1.0 for the default infectious state some ratio
    /// compared to this state.
    fn contagion_odds(&self) -> Real;

    /// Return true if one agent can contaminate the other. This must return true
    /// if contagion is, in principle, possible. Further external restrictions
    /// (like, e.g., physical distance) may make the infection impossible, but
    /// this should be treated later in the pipeline.
    fn can_contaminate(&self, other: &Self) -> bool {
        self.is_contagious() && other.is_susceptible()
    }

    /// Return a copy of agent after interacting with other assuming that
    /// contamination should occur. This method return None if `other` is not able
    /// to contaminate `self`
    fn contaminated_from(&self, other: &Self) -> Option<Self> {
        if other.can_contaminate(self) {
            let mut new = self.clone();
            new.transfer_contamination_from(other);
            return Some(new);
        }
        return None;
    }

    /// Transfer contamination state from other and return a boolean telling if
    /// the contamination occurred or not. This method should force contamination
    /// even when it is not clinically possible (e.g., self is recovered).
    /// It is used by contaminate_from(), which checks basic clinical plausibility
    /// and uses this method to transfer contamination to a susceptible agent
    /// if contamination is possible.
    ///
    /// This method should not simply clone the contamination state from `other`
    /// but rather modify self to the proper state that describes the start
    /// of an infection. For instance, in a model like SEIR, if other is in some
    /// infected state, like `SEIR::Recovered(params)`, self will be modified
    /// to `SEIR::Exposed(params)`, independently from its current state.
    fn transfer_contamination_from(&mut self, other: &Self) -> bool;

    /// Contaminate `self` from `other` if contamination is plausible.
    fn contaminate_from(&mut self, other: &Self) -> bool {
        if other.can_contaminate(self) {
            self.transfer_contamination_from(other);
            return true;
        }
        return false;
    }

    /// Contaminate `self` from `other` with probability `prob`, if
    /// contamination is plausible.
    fn contaminate_from_prob<R: Rng>(&mut self, other: &Self, prob: Real, rng: &mut R) -> bool {
        if other.can_contaminate(self) && rng.gen_bool(prob) {
            self.transfer_contamination_from(other);
            return true;
        }
        return false;
    }

    /// Return true if agent is dead from disease.
    fn is_dead(&self) -> bool {
        self.index() == Self::D
    }

    /// Return true if agent is alive. This is just a negation of is_dead and
    /// traits should generally implement only the former.
    fn is_alive(&self) -> bool {
        !self.is_dead()
    }
}

macro_rules! is_state {
    ($name:ident, index=$idx:ident) => {
        paste! {
            #[doc = "Return true if agent is in the  `" $name "` state."]
            fn [<is_ $name>](&self) -> bool {
                self.index() == Self::$idx
            }
        }
    };
    ($name:ident) => {
        paste! {
            #[doc = "Return true if agent is in the  `" $name "` state."]
            fn [<is_ $name>](&self) -> bool;
        }
    };
    ($name:ident, $expr:expr) => {
        paste! {
            #[doc = "Return true if agent is in the  `" $name "` state."]
            fn [<is_ $name>](&self) -> bool { $expr }
        }
    };
    ($name:ident, |$var:ident| $expr:expr) => {
        paste! {
            #[doc = "Return true if agent is in the  `" $name "` state."]
            fn [<is_ $name>](&self) -> bool {
                let $var = self; $expr
             }
        }
    };
}

/// The classical Susceptible, Exposed, Infectious, Recovered model. This crate
/// also assumes a distinct Dead state which is usually grouped with Recovered
/// in the classical SIR model.
pub trait SEIRLike: EpiModel {
    const E: usize;
    const I: usize;
    const R: usize;

    is_state!(exposed, index = E);
    is_state!(infectious, index = I);
    is_state!(recovered, index = R);
    is_state!(dead, index = D);

    /// Force exposure of agent to infection using the given clinical
    /// parameters.
    fn expose(&mut self, with: &Self::Clinical) {
        self.infect(with)
    }

    /// Force exposure of agent to infection using the default clinical
    /// parameters, if they exist.
    fn expose_default(&mut self)
    where
        Self::Clinical: Default,
    {
        self.expose(&Self::Clinical::default())
    }

    /// Force exposure of agent to infection using the given clinical
    /// parameters. Differently from expose() this method always puts agent
    /// into a infectious state.
    fn infect(&mut self, with: &Self::Clinical);

    /// Like infect(), but uses the default infection parameters.
    fn infect_default(&mut self)
    where
        Self::Clinical: Default,
    {
        self.infect(&Self::Clinical::default())
    }
}

/// Extends the SEIR with asymptomatic agents and a severity model in which
/// agents may require healthcare.
///
/// Asymptomatics do not develop symptoms and severity of disease never
/// aggravates, but they may transmit it to other agents (even if with a
/// reduced transmissibility).
///
/// SEICHAR consider severity levels:
///
/// Severe/Hospitalized:
///     Agents that require access to simple healthcare facilities.
/// Critical/ICU:
///     Cases that are severe enough to require intensive care.    
pub trait SEICHARLike: SEIRLike {
    const C: usize;
    const H: usize;
    const A: usize;
    is_state!(asymptomatic, index = A);
    is_state!(severe, index = H);
    is_state!(critical, index = C);
}

////////////////////////////////////////////////////////////////////////////////
// Trait implementations
////////////////////////////////////////////////////////////////////////////////
