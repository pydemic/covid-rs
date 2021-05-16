use rand::Rng;

use crate::prelude::{Age, Params, SimpleSEAIR, SimpleSEICHAR, SimpleSEIR, SimpleSIR, SIR};
use std::fmt::Debug;

/// Minimal bounds on agent States.
pub trait State: Send + Clone + PartialEq + Debug + Default {} // Sync?
pub trait World: Send + Clone + PartialEq + Debug + Default {} // Sync?

/// A trait for agents that have an age field. Many epidemiological models are
/// age-sensitive and it is useful to isolate this property as trait to implement
/// generic functionality associated with processing ages.
pub trait HasAge: State {
    /// Agent's age.
    fn age(&self) -> Age;

    /// Set age with given value.
    fn set_age(&mut self, value: Age) -> &mut Self;
}

/// A trait for objects that have an compartment field with a SIR value.
pub trait HasCompartiment: State {
    type CompartmentModel: SIR;

    /// Return the epidemiological compartment.
    fn compartment(&self) -> Self::CompartmentModel;

    /// Set value of the epidemiological state.
    fn set_compartment(&mut self, value: Self::CompartmentModel) -> &mut Self;
}

/// A trait for objects that can be deterministically updated in the given
/// World.
pub trait Update<W: World>: State {
    fn update(&mut self, world: &W);
}

/// A trait for objects that can be stochastically updated in the given
/// World. Users must pass a random number generator in order to update
// this objectobject
pub trait StochasticUpdate<W: World, R: Rng>: State {
    fn update_random(&mut self, world: &W, rng: &mut R);
}

/////////////////////////////////////////////////////////////////////////////
// Default trait implementations
/////////////////////////////////////////////////////////////////////////////

impl State for () {}

impl<W: World> Update<W> for () {
    fn update(&mut self, _world: &W) {}
}

impl<W: World, R: Rng> StochasticUpdate<W, R> for () {
    fn update_random(&mut self, _world: &W, _: &mut R) {}
}

// Epidemic models //////////////////////////////////////////////////////////
impl<R: Rng> StochasticUpdate<Params, R> for SimpleSIR {
    fn update_random(&mut self, params: &Params, rng: &mut R) {
        match self {
            Self::Infectious => {
                if rng.gen_bool(params.infectious_transition_prob()) {
                    *self = Self::Recovered
                }
            }
            _ => (),
        }
    }
}

impl<R: Rng> StochasticUpdate<Params, R> for SimpleSEIR {
    fn update_random(&mut self, params: &Params, rng: &mut R) {
        match self {
            Self::Exposed => {
                if rng.gen_bool(params.incubation_transition_prob()) {
                    *self = Self::Infectious
                }
            }
            Self::Infectious => {
                if rng.gen_bool(params.infectious_transition_prob()) {
                    *self = Self::Recovered
                }
            }
            _ => (),
        }
    }
}

impl<R: Rng> StochasticUpdate<Params, R> for SimpleSEAIR {
    fn update_random(&mut self, params: &Params, rng: &mut R) {
        match self {
            Self::Exposed => {
                if rng.gen_bool(params.incubation_transition_prob()) {
                    if rng.gen_bool(params.prob_asymptomatic(40)) {
                        *self = Self::Asymptomatic
                    } else {
                        *self = Self::Infectious
                    }
                }
            }
            Self::Asymptomatic => {
                if rng.gen_bool(params.infectious_transition_prob()) {
                    *self = Self::Recovered
                }
            }
            Self::Infectious => {
                if rng.gen_bool(params.infectious_transition_prob()) {
                    *self = Self::Recovered
                }
            }
            _ => (),
        }
    }
}

impl<R: Rng> StochasticUpdate<Params, R> for SimpleSEICHAR {
    fn update_random(&mut self, params: &Params, rng: &mut R) {
        let age = 40; // TODO
        match self {
            Self::Exposed => {
                if rng.gen_bool(params.incubation_transition_prob()) {
                    if rng.gen_bool(params.prob_asymptomatic(age)) {
                        *self = Self::Asymptomatic
                    } else {
                        *self = Self::Infectious
                    }
                }
            }
            Self::Asymptomatic => {
                if rng.gen_bool(params.infectious_transition_prob()) {
                    *self = Self::Recovered
                }
            }
            Self::Infectious => {
                if rng.gen_bool(params.infectious_transition_prob()) {
                    if rng.gen_bool(params.prob_severe(age)) {
                        *self = Self::Severe
                    } else {
                        *self = Self::Recovered
                    }
                }
            }
            Self::Severe => {
                if rng.gen_bool(params.severe_transition_prob()) {
                    if rng.gen_bool(params.prob_critical(age)) {
                        *self = Self::Critical
                    } else {
                        *self = Self::Recovered
                    }
                }
            }
            Self::Critical => {
                if rng.gen_bool(params.critical_transition_prob()) {
                    if rng.gen_bool(params.prob_death(age)) {
                        *self = Self::Dead
                    } else {
                        *self = Self::Recovered
                    }
                }
            }
            _ => (),
        }
    }
}

impl<R, T> StochasticUpdate<Params, R> for T
where
    R: Rng,
    T: HasCompartiment,
    T::CompartmentModel: StochasticUpdate<Params, R>,
{
    fn update_random(&mut self, params: &Params, rng: &mut R) {
        let mut value = self.compartment();
        value.update_random(params, rng);
        self.set_compartment(value);
    }
}