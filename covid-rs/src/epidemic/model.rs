use crate::{
    prelude::*,
    sim::{Agent, State},
};
use serde::{Deserialize, Serialize};

pub trait Enumerable {
    const CARDINALITY: usize;
    fn index(&self) -> usize;
}

pub trait SIR: Enumerable + Default {
    const S: usize;
    const I: usize;
    const R: usize;
    const D: usize;

    fn is_susceptible(&self) -> bool {
        self.index() == Self::S
    }
    fn is_infectious(&self) -> bool {
        self.index() == Self::I
    }
    fn is_recovered(&self) -> bool {
        self.index() == Self::R
    }
    fn is_infecting(&self) -> bool {
        self.is_infectious()
    }
    fn is_dead(&self) -> bool {
        self.index() == Self::D
    }
    fn expose(&mut self) {
        self.infect()
    }
    fn contaminated_from(&self, other: &Self) -> Option<Self>;
    fn infect(&mut self);
}

pub trait SEIR: SIR {
    const E: usize;

    fn is_exposed(&self) -> bool {
        self.index() == Self::E
    }
}

pub trait SEAIR: SEIR {
    const A: usize;

    fn is_asymptomatic(&self) -> bool {
        self.index() == Self::A
    }
}

pub trait SEICHAR: SEAIR {
    const C: usize;
    const H: usize;

    fn is_severe(&self) -> bool {
        self.index() == Self::H
    }
    
    fn is_critical(&self) -> bool {
        self.index() == Self::C
    }
}

/* SIR MODEL ******************************************************************/

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub enum SimpleSIR {
    Susceptible,
    Infectious,
    Recovered,
    Dead,
}

impl Enumerable for SimpleSIR {
    const CARDINALITY: usize = 4;
    fn index(&self) -> usize {
        match self {
            Self::Susceptible => Self::S,
            Self::Infectious => Self::I,
            Self::Recovered => Self::R,
            Self::Dead => Self::D,
        }
    }
}

impl Default for SimpleSIR {
    fn default() -> Self {
        Self::Susceptible
    }
}

impl SIR for SimpleSIR {
    const S: usize = 0;
    const I: usize = 1;
    const R: usize = 2;
    const D: usize = 3;

    fn contaminated_from(&self, other: &Self) -> Option<Self> {
        (other.is_infecting() && self.is_susceptible()).then(|| Self::Infectious)
    }

    fn infect(&mut self) {
        *self = Self::Infectious
    }
}

impl State for SimpleSIR {}

/* SEIR MODEL *****************************************************************/
#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub enum SimpleSEIR {
    Susceptible,
    Exposed,
    Infectious,
    Recovered,
    Dead,
}

impl Enumerable for SimpleSEIR {
    const CARDINALITY: usize = 5;
    fn index(&self) -> usize {
        match self {
            Self::Susceptible => Self::S,
            Self::Exposed => Self::E,
            Self::Infectious => Self::I,
            Self::Recovered => Self::R,
            Self::Dead => Self::D,
        }
    }
}

impl Default for SimpleSEIR {
    fn default() -> Self {
        Self::Susceptible
    }
}

impl SIR for SimpleSEIR {
    const S: usize = 0;
    const I: usize = 2;
    const R: usize = 3;
    const D: usize = 4;

    fn contaminated_from(&self, other: &Self) -> Option<Self> {
        (other.is_infecting() && self.is_susceptible()).then(|| Self::Exposed)
    }

    fn infect(&mut self) {
        *self = Self::Infectious
    }

    fn expose(&mut self) {
        *self = Self::Exposed
    }
}

impl SEIR for SimpleSEIR {
    const E: usize = 1;
}

impl State for SimpleSEIR {}

/* SEAIR MODEL *****************************************************************/
#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub enum SimpleSEAIR {
    Susceptible,
    Exposed,
    Asymptomatic,
    Infectious,
    Recovered,
}

impl Enumerable for SimpleSEAIR {
    const CARDINALITY: usize = 5;
    fn index(&self) -> usize {
        match self {
            Self::Susceptible => Self::S,
            Self::Exposed => Self::E,
            Self::Asymptomatic => Self::A,
            Self::Infectious => Self::I,
            Self::Recovered => Self::R,
        }
    }
}

impl Default for SimpleSEAIR {
    fn default() -> Self {
        Self::Susceptible
    }
}

impl SIR for SimpleSEAIR {
    const S: usize = 0;
    const I: usize = 3;
    const R: usize = 4;
    const D: usize = 5;

    fn contaminated_from(&self, other: &Self) -> Option<Self> {
        (other.is_infecting() && self.is_susceptible()).then(|| Self::Exposed)
    }

    fn is_infecting(&self) -> bool {
        self.is_asymptomatic() || self.is_infectious()
    }

    fn infect(&mut self) {
        *self = Self::Infectious
    }

    fn expose(&mut self) {
        *self = Self::Exposed
    }
}

impl SEIR for SimpleSEAIR {
    const E: usize = 1;
}

impl SEAIR for SimpleSEAIR {
    const A: usize = 2;
}

impl State for SimpleSEAIR {}

/* SEICHAR MODEL **************************************************************/
#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub enum SimpleSEICHAR {
    Susceptible,
    Exposed,
    Infectious,
    Critical,
    Severe,
    Asymptomatic,
    Recovered,
    Dead,
}

impl Enumerable for SimpleSEICHAR {
    const CARDINALITY: usize = 7;
    fn index(&self) -> usize {
        match self {
            Self::Susceptible => Self::S,
            Self::Exposed => Self::E,
            Self::Critical => Self::C,
            Self::Severe => Self::H,
            Self::Asymptomatic => Self::A,
            Self::Infectious => Self::I,
            Self::Recovered => Self::R,
            Self::Dead => Self::D,
        }
    }
}

impl Default for SimpleSEICHAR {
    fn default() -> Self {
        Self::Susceptible
    }
}

impl SIR for SimpleSEICHAR {
    const S: usize = 0;
    const I: usize = 2;
    const R: usize = 6;
    const D: usize = 7;

    fn contaminated_from(&self, other: &Self) -> Option<Self> {
        (other.is_infecting() && self.is_susceptible()).then(|| Self::Exposed)
    }

    fn infect(&mut self) {
        *self = Self::Infectious
    }

    fn expose(&mut self) {
        *self = Self::Exposed
    }
}

impl SEIR for SimpleSEICHAR {
    const E: usize = 1;
}

impl SEAIR for SimpleSEICHAR {
    const A: usize = 5;
}

impl SEICHAR for SimpleSEICHAR {
    const C: usize = 3;
    const H: usize = 4;
}

impl State for SimpleSEICHAR {}

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

impl Enumerable for VariantSEICHAR {
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

impl SIR for VariantSEICHAR {
    const S: usize = 0;
    const I: usize = 2;
    const R: usize = 6;
    const D: usize = 7;

    fn contaminated_from(&self, other: &Self) -> Option<Self> {
        if self.is_susceptible() {
            return match other {
                Self::Infectious(v) => Some(Self::Exposed(*v)),
                Self::Asymptomatic(v) => Some(Self::Exposed(*v)),
                _ => None,
            };
        }
        return None;
    }

    fn infect(&mut self) {
        *self = Self::Infectious(Default::default())
    }

    fn expose(&mut self) {
        *self = Self::Exposed(Default::default())
    }
}

impl SEIR for VariantSEICHAR {
    const E: usize = 1;
}

impl SEAIR for VariantSEICHAR {
    const A: usize = 5;
}

impl SEICHAR for VariantSEICHAR {
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

impl<M: Enumerable> Enumerable for Agent<M> {
    const CARDINALITY: usize = M::CARDINALITY;

    fn index(&self) -> usize {
        self.state.index()
    }
}

impl<M: SIR + Enumerable> SIR for Agent<M> {
    const S: usize = M::S;
    const I: usize = M::I;
    const R: usize = M::R;
    const D: usize = M::D;

    fn contaminated_from(&self, other: &Self) -> Option<Self> {
        self.state
            .contaminated_from(&other.state)
            .map(|state| Agent { id: self.id, state })
    }

    fn infect(&mut self) {
        self.state.infect()
    }

    fn expose(&mut self) {
        self.state.expose()
    }

    fn is_susceptible(&self) -> bool {
        self.state.is_susceptible()
    }

    fn is_infectious(&self) -> bool {
        self.state.is_infectious()
    }

    fn is_recovered(&self) -> bool {
        self.state.is_recovered()
    }

    fn is_infecting(&self) -> bool {
        self.state.is_infecting()
    }

    fn is_dead(&self) -> bool {
        self.state.is_dead()
    }
}
