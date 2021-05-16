use crate::{
    epidemic::*,
    prelude::*,
    sim::{HasAge, State, StochasticUpdate},
};
use getset::*;
use rand::prelude::*;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Infect {
    Natural,
    ForceExposed,
    ForceInfectious,
}

impl From<Option<bool>> for Infect {
    fn from(opt: Option<bool>) -> Infect {
        match opt {
            Some(true) => Infect::ForceInfectious,
            Some(false) => Infect::ForceExposed,
            None => Infect::Natural,
        }
    }
}

impl From<bool> for Infect {
    fn from(opt: bool) -> Infect {
        match opt {
            true => Infect::ForceInfectious,
            false => Infect::ForceExposed,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default, CopyGetters, Setters)]
#[getset(get_copy = "pub")]
pub struct Ag {
    #[getset(set = "pub")]
    age: Age,
    state: VariantSEICHAR,
    state_t: Time,
    vaccine: bool,
    vaccine_t: Time,
    secondary_infections: usize,
}

impl Ag {
    /// Create new agent with the given age.
    pub fn new(age: Age) -> Self {
        Ag {
            age,
            ..Ag::default()
        }
    }

    /// Main update method disconsidering interactions between agents.
    pub fn update<R: Rng>(&mut self, rng: &mut R, params_baseline: &Params, params_voc: &Params) {
        self.state_t += 1;
        self.vaccine_t += 1;
        match self.state {
            VariantSEICHAR::Susceptible => {}
            VariantSEICHAR::Exposed(v) => {
                let params = v.select(params_baseline, params_voc);
                if rng.gen_bool(params.incubation_transition_prob()) {
                    if rng.gen_bool(params.prob_asymptomatic(self.age)) {
                        self.set_status(VariantSEICHAR::Asymptomatic(v));
                    } else {
                        self.set_status(VariantSEICHAR::Infectious(v));
                    }
                }
            }
            VariantSEICHAR::Infectious(v) => {
                let params = v.select(params_baseline, params_voc);
                if rng.gen_bool(params.infectious_transition_prob()) {
                    if rng.gen_bool(params.prob_severe(self.age)) {
                        self.set_status(VariantSEICHAR::Severe(v));
                    } else {
                        self.recover();
                    }
                }
            }
            VariantSEICHAR::Critical(v) => {
                let params = v.select(params_baseline, params_voc);
                if rng.gen_bool(params.critical_transition_prob()) {
                    if rng.gen_bool(params.prob_death(self.age)) {
                        self.set_status(VariantSEICHAR::Dead(v));
                    } else {
                        self.recover();
                    }
                }
            }
            VariantSEICHAR::Severe(v) => {
                let params = v.select(params_baseline, params_voc);
                if rng.gen_bool(params.severe_transition_prob()) {
                    if rng.gen_bool(params.prob_critical(self.age)) {
                        self.set_status(VariantSEICHAR::Critical(v));
                    } else {
                        self.recover();
                    }
                }
            }
            VariantSEICHAR::Asymptomatic(v) => {
                let params = v.select(params_baseline, params_voc);
                if rng.gen_bool(params.infectious_transition_prob()) {
                    self.recover();
                }
            }
            VariantSEICHAR::Recovered(_) => {}
            VariantSEICHAR::Dead(_) => {}
        };
    }

    /// Set the infection state of agent.
    pub fn set_status(&mut self, state: VariantSEICHAR) {
        if state != self.state {
            self.state = state;
            self.state_t = 0;
        }
    }
    /// Change state to Susceptible, recovering from the current infection.
    pub fn recover(&mut self) {
        match self.variant() {
            Some(v) => {
                self.set_status(VariantSEICHAR::Recovered(v));
            }
            None => {}
        }
    }

    /// Increment the infection count.
    pub fn register_secondary_infection(&mut self) {
        self.secondary_infections += 1;
    }

    /// Infect/expose individual with variant, changing its status.
    /// Return true when infection occurs.
    pub fn contaminate(&mut self, variant: Variant, infect: Infect) -> bool {
        match infect {
            Infect::ForceInfectious => self.set_status(VariantSEICHAR::Infectious(variant)),
            Infect::ForceExposed => self.set_status(VariantSEICHAR::Exposed(variant)),
            Infect::Natural => {
                if !self.is_susceptible_to(variant) {
                    return false;
                }
                self.set_status(VariantSEICHAR::Exposed(variant));
            }
        }
        return true;
    }

    /// Query if agent is suspectible to other infections.
    pub fn is_susceptible(&self) -> bool {
        self.state == VariantSEICHAR::Susceptible
    }

    /// Query if agent is suspectible to infections from the given variant.
    pub fn is_susceptible_to(&self, _variant: Variant) -> bool {
        self.state == VariantSEICHAR::Susceptible
    }

    /// Query if agent can infect other agents. This happens when agent is in
    /// the infectious/asymptomatic comparments or as severe/critical with
    /// poor healthcare conditions.
    pub fn is_infecting(&self) -> bool {
        self.active_variant().is_some()
    }

    /// Some(Variant) by which the agent is infectious or None.
    pub fn active_variant(&self) -> Option<Variant> {
        match self.state {
            VariantSEICHAR::Infectious(v) => Some(v),
            VariantSEICHAR::Asymptomatic(v) => Some(v),
            _ => None,
        }
    }

    /// Some(Variant) that infects/infected agent or None.
    pub fn variant(&self) -> Option<Variant> {
        match self.state {
            VariantSEICHAR::Susceptible => None,
            VariantSEICHAR::Exposed(v) => Some(v),
            VariantSEICHAR::Asymptomatic(v) => Some(v),
            VariantSEICHAR::Infectious(v) => Some(v),
            VariantSEICHAR::Severe(v) => Some(v),
            VariantSEICHAR::Critical(v) => Some(v),
            VariantSEICHAR::Recovered(v) => Some(v),
            VariantSEICHAR::Dead(v) => Some(v),
        }
    }
}

impl State for Ag {}

impl Enumerable for Ag {
    const CARDINALITY: usize = VariantSEICHAR::CARDINALITY;

    fn index(&self) -> usize {
        self.state.index()
    }
}

impl SIR for Ag {
    const S: usize = VariantSEICHAR::S;
    const I: usize = VariantSEICHAR::I;
    const R: usize = VariantSEICHAR::R;
    const D: usize = VariantSEICHAR::D;

    fn contaminated_from(&self, other: &Self) -> Option<Self> {
        self.state.contaminated_from(&other.state).map(|st| {
            let mut new = self.clone();
            new.state = st;
            return new;
        })
    }

    fn infect(&mut self) {
        self.state.infect()
    }

    fn expose(&mut self) {
        self.state.expose()
    }
}

impl SEIR for Ag {
    const E: usize = VariantSEICHAR::E;
}

impl SEAIR for Ag {
    const A: usize = VariantSEICHAR::A;
}

impl SEICHAR for Ag {
    const C: usize = VariantSEICHAR::C;
    const H: usize = VariantSEICHAR::H;
}

impl HasAge for Ag {
    fn age(&self) -> Age {
        self.age
    }
    fn set_age(&mut self, value: Age) -> &mut Self {
        self.age = value;
        return self;
    }
}

impl StochasticUpdate<Params> for Ag {
    fn update_random<R: Rng>(&mut self, params: &Params, rng: &mut R) {
        self.update(rng, params, params)
    }
}
