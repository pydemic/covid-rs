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

/// The classical Susceptible, Infectious, Recovered model. This crate also
/// assumes a distinct Dead state which is usually grouped with Recovered in the
/// classical SIR model.
pub trait SIRLike: EpiModel {
    const I: usize;
    const R: usize;

    is_state!(infectious, index = I);
    is_state!(recovered, index = R);
    is_state!(dead, index = D);
    is_state!(exposed);

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

/// Extends the SIR model with an intermediary incubation time in the Exposed
/// state in which a contaminated agent is not contagious yet.
pub trait SEIRLike: SIRLike {
    const E: usize;
}

/// Extends the SEIR model with asymptomatic agents. Asymptomatics do not
/// develop symptoms and severity of disease never aggravates, but they may
/// transmit it to other agents (even if with a reduced transmissibility).
pub trait SEAIRLike: SEIRLike {
    const A: usize;
    is_state!(asymptomatic, index = A);
}

/// Extends the SEAIR with a severity model in which agents start requiring
/// healthcare. SEICHAR consider two levels:
///
/// Severe/Hospitalized:
///     Agents that require access to simple healthcare facilities.
/// Critical/ICU:
///     Cases that are severe enough to require intensive care.    
pub trait SEICHARLike: SEAIRLike {
    const C: usize;
    const H: usize;
    is_state!(severe, index = H);
    is_state!(critical, index = C);
}

////////////////////////////////////////////////////////////////////////////////
// SEICHAR MODEL Implementation
////////////////////////////////////////////////////////////////////////////////

/*

/* SEICHAR/Variant model ******************************************************/
#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub enum VariantSEICHAR {
    Susceptible,
    Exposed(Variant),
    Asymptomatic(Variant),
    Infectious(Variant),
    Severe(Variant),
    Critical(Variant),
    Recovered(Variant),
    Dead(Variant),
}

impl VariantSEICHAR {
    pub const HEADER_SHORT: [&'static str; 8] = ["S", "E", "I", "C", "H", "A", "R", "D"];
    pub const HEADER_LONG: [&'static str; 8] = [
        "susceptible",
        "exposed",
        "infectious",
        "critical",
        "severe",
        "asymptomatic",
        "recovered",
        "dead",
    ];

    pub fn csv_header() -> String {
        Self::HEADER_SHORT.join(",")
    }

    pub fn csv(self) -> String {
        let ch = Self::HEADER_SHORT[self.index()];
        return match self {
            VariantSEICHAR::Susceptible => format!("{}", ch),
            VariantSEICHAR::Exposed(v) => format!("{}{}", ch, v.csv()),
            VariantSEICHAR::Infectious(v) => format!("{}{}", ch, v.csv()),
            VariantSEICHAR::Severe(v) => format!("{}{}", ch, v.csv()),
            VariantSEICHAR::Critical(v) => format!("{}{}", ch, v.csv()),
            VariantSEICHAR::Asymptomatic(v) => format!("{}{}", ch, v.csv()),
            VariantSEICHAR::Recovered(v) => format!("{}{}", ch, v.csv()),
            VariantSEICHAR::Dead(v) => format!("{}{}", ch, v.csv()),
        };
    }
}

impl EpiModel for VariantSEICHAR {
    const CARDINALITY: usize = 8;

    fn index(&self) -> usize {
        match self {
            Self::Susceptible => Self::S,
            Self::Exposed(_) => Self::E,
            Self::Infectious(_) => Self::I,
            Self::Critical(_) => Self::C,
            Self::Severe(_) => Self::H,
            Self::Asymptomatic(_) => Self::A,
            Self::Recovered(_) => Self::R,
            Self::Dead(_) => Self::D,
        }
    }
}

impl SIRLike for VariantSEICHAR {
    const S: usize = 0;
    const I: usize = 2;
    const R: usize = 6;
    const D: usize = 7;

    // fn contaminated_from(&self, other: &Self) -> Option<Self> {
    //     if self.is_susceptible() {
    //         return match other {
    //             Self::Infectious(v) => Some(Self::Exposed(*v)),
    //             Self::Asymptomatic(v) => Some(Self::Exposed(*v)),
    //             _ => None,
    //         };
    //     }
    //     return None;
    // }

    fn infect(&mut self) {
        *self = Self::Infectious(Default::default())
    }

    fn expose(&mut self) {
        *self = Self::Exposed(Default::default())
    }
}

impl SEIRLike for VariantSEICHAR {
    const E: usize = 1;
}

impl SEAIRLike for VariantSEICHAR {
    const A: usize = 5;
}

impl SEICHARLike for VariantSEICHAR {
    const C: usize = 3;
    const H: usize = 4;
}

impl Default for VariantSEICHAR {
    fn default() -> Self {
        VariantSEICHAR::Susceptible
    }
}

impl State for VariantSEICHAR {}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StateStats<T> {
    pub susceptible: T,
    pub exposed: T,
    pub infectious: T,
    pub critical: T,
    pub severe: T,
    pub asymptomatic: T,
    pub recovered: T,
    pub dead: T,
}

impl<T> StateStats<T> {
    pub fn map<S, F>(&self, f: F) -> StateStats<S>
    where
        F: Fn(T) -> S,
        T: Clone,
    {
        StateStats {
            susceptible: f(self.susceptible.clone()),
            exposed: f(self.exposed.clone()),
            infectious: f(self.infectious.clone()),
            critical: f(self.critical.clone()),
            severe: f(self.severe.clone()),
            asymptomatic: f(self.asymptomatic.clone()),
            recovered: f(self.recovered.clone()),
            dead: f(self.dead.clone()),
        }
    }
}

impl<M: EpiModel> EpiModel for Agent<M> {
    const CARDINALITY: usize = M::CARDINALITY;

    fn index(&self) -> usize {
        self.state.index()
    }
}

impl<M: SIRLike + EpiModel> SIRLike for Agent<M> {
    const S: usize = M::S;
    const I: usize = M::I;
    const R: usize = M::R;
    const D: usize = M::D;

    // fn contaminated_from(&self, other: &Self) -> Option<Self> {
    //     self.state
    //         .contaminated_from(&other.state)
    //         .map(|state| Agent { id: self.id, state })
    // }

    // fn infect(&mut self) {
    //     self.state.infect()
    // }

    // fn expose(&mut self) {
    //     self.state.expose()
    // }

    fn is_susceptible(&self) -> bool {
        self.state.is_susceptible()
    }

    fn is_infectious(&self) -> bool {
        self.state.is_infectious()
    }

    fn is_recovered(&self) -> bool {
        self.state.is_recovered()
    }

    // fn is_infecting(&self) -> bool {
    //     self.state.is_infecting()
    // }

    fn is_dead(&self) -> bool {
        self.state.is_dead()
    }
}
*/
