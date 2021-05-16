use crate::{
    prelude::{Age, AgeDistribution10, AgeParam, ForAge, Real},
    sim::World,
};
use getset::*;
use paste::paste;
use serde::{Deserialize, Deserializer, Serialize};

// Default param values
const PROB_ASYMPTOMATIC: Real = 0.42;
const PROB_SEVERE: Real = 0.18;
const PROB_CRITICAL: Real = 0.22;
const PROB_DEATH: Real = 0.42;
const INCUBATION_PERIOD: Real = 3.69;
const INFECTIOUS_PERIOD: Real = 3.47;
const SEVERE_PERIOD: Real = 7.19;
const CRITICAL_PERIOD: Real = 17.50 - 7.19;

#[derive(CopyGetters, Getters, Setters, Debug, PartialEq, Copy, Clone, Serialize)]
#[serde(default)]
pub struct Params {
    #[getset(get_copy = "pub")]
    incubation_period: Real,

    #[getset(get_copy = "pub")]
    infectious_period: Real,

    #[getset(get_copy = "pub")]
    severe_period: Real,

    #[getset(get_copy = "pub")]
    critical_period: Real,

    #[getset(get_copy = "pub", set = "pub")]
    infectiousness: Real,

    /// Probability of transition exposed -> ? in a single day.
    #[getset(get_copy = "pub")]
    #[serde(skip)]
    incubation_transition_prob: Real,

    /// Probability of transition infectious -> ? in a single day.
    #[getset(get_copy = "pub")]
    #[serde(skip)]
    infectious_transition_prob: Real,

    /// Probability of transition severe -> ? in a single day.
    #[getset(get_copy = "pub")]
    #[serde(skip)]
    severe_transition_prob: Real,

    /// Probability of transition critical -> ? in a single day.
    #[getset(get_copy = "pub")]
    #[serde(skip)]
    critical_transition_prob: Real,

    /// Probability that an exposed person becomes asymptomatic
    prob_asymptomatic: AgeParam,

    /// Probability that an infections agent evolves to severe
    prob_severe: AgeParam,

    /// Probability that a severe patient aggravates to critical
    prob_critical: AgeParam,

    /// Probability that a critical patient dies
    prob_death: AgeParam,
}

impl World for Params {}

macro_rules! value_prop {
    ($x:ident) => {
        paste! {
            pub fn $x(&self, age: Age) -> Real {
                self.$x.for_age(age)
            }
            pub fn [<set_ $x>](&mut self, value: Real) {
                self.$x = AgeParam::Scalar(value);
            }
            pub fn [<set_ $x _distrib>](&mut self, value: AgeDistribution10) {
                self.$x = AgeParam::Distribution(value);
            }
        }
    };
}

impl Params {
    value_prop!(prob_asymptomatic);
    value_prop!(prob_severe);
    value_prop!(prob_critical);
    value_prop!(prob_death);

    /// Set mean incubation period and update transition probability
    pub fn set_incubation_period(&mut self, value: Real) -> &mut Self {
        self.incubation_period = value;
        self.incubation_transition_prob = 1.0 - (-1. / value).exp();
        return self;
    }

    /// Set mean infectious period and update transition probability
    pub fn set_infectious_period(&mut self, value: Real) -> &mut Self {
        self.infectious_period = value;
        self.infectious_transition_prob = 1.0 - (-1. / value).exp();
        return self;
    }

    /// Set mean severe period and update transition probability
    pub fn set_severe_period(&mut self, value: Real) -> &mut Self {
        self.severe_period = value;
        self.severe_transition_prob = 1.0 - (-1. / value).exp();
        return self;
    }

    /// Set mean critical period and update transition probability
    pub fn set_critical_period(&mut self, value: Real) -> &mut Self {
        self.critical_period = value;
        self.critical_transition_prob = 1.0 - (-1. / value).exp();
        return self;
    }

    /// Merge infectious and incubation period. This tends to make SIR models
    /// behave more similarly to their SEIR counterparts.
    pub fn merge_incubation_period(&mut self) -> &mut Self {
        self.set_infectious_period(self.infectious_period + self.incubation_period);
        return self.set_incubation_period(0.0);
    }

    /// Probability of death for cases
    pub fn case_fatality_ratio(&self, age: Age) -> Real {
        self.prob_death(age) * self.prob_critical(age) * self.prob_severe(age)
    }

    /// Probability of death for all infections (symptomatic or not)
    pub fn infection_fatality_ratio(&self, age: Age) -> Real {
        self.case_fatality_ratio(age) * self.prob_asymptomatic(age)
    }
}

impl Default for Params {
    fn default() -> Self {
        let mut new = Params {
            incubation_period: 0.0,
            infectious_period: 0.0,
            severe_period: 0.0,
            critical_period: 0.0,
            incubation_transition_prob: 0.0,
            infectious_transition_prob: 0.0,
            severe_transition_prob: 0.0,
            critical_transition_prob: 0.0,
            infectiousness: 1.0,
            prob_asymptomatic: AgeParam::Scalar(PROB_ASYMPTOMATIC),
            prob_severe: AgeParam::Scalar(PROB_SEVERE),
            prob_critical: AgeParam::Scalar(PROB_CRITICAL),
            prob_death: AgeParam::Scalar(PROB_DEATH),
        };

        new.set_incubation_period(INCUBATION_PERIOD);
        new.set_infectious_period(INFECTIOUS_PERIOD);
        new.set_severe_period(SEVERE_PERIOD);
        new.set_critical_period(CRITICAL_PERIOD);
        return new;
    }
}

#[derive(Debug, PartialEq, Copy, Clone, Deserialize)]
#[serde(default)]
struct _Params {
    incubation_period: Real,
    infectious_period: Real,
    severe_period: Real,
    infectiousness: Real,
    critical_period: Real,
    prob_asymptomatic: AgeParam,
    prob_severe: AgeParam,
    prob_critical: AgeParam,
    prob_death: AgeParam,
}

impl Default for _Params {
    fn default() -> Self {
        _Params {
            incubation_period: INCUBATION_PERIOD,
            infectious_period: INFECTIOUS_PERIOD,
            severe_period: SEVERE_PERIOD,
            infectiousness: 1.0,
            critical_period: CRITICAL_PERIOD,
            prob_asymptomatic: AgeParam::Scalar(PROB_ASYMPTOMATIC),
            prob_severe: AgeParam::Scalar(PROB_SEVERE),
            prob_critical: AgeParam::Scalar(PROB_CRITICAL),
            prob_death: AgeParam::Scalar(PROB_DEATH),
        }
    }
}

impl From<_Params> for Params {
    fn from(p: _Params) -> Params {
        let mut new = Params {
            incubation_period: 0.0,
            infectious_period: 0.0,
            severe_period: 0.0,
            critical_period: 0.0,
            incubation_transition_prob: 0.0,
            infectious_transition_prob: 0.0,
            severe_transition_prob: 0.0,
            critical_transition_prob: 0.0,
            infectiousness: p.infectiousness,
            prob_asymptomatic: p.prob_asymptomatic,
            prob_severe: p.prob_severe,
            prob_critical: p.prob_critical,
            prob_death: p.prob_death,
        };
        new.set_incubation_period(p.incubation_period);
        new.set_infectious_period(p.infectious_period);
        new.set_severe_period(p.severe_period);
        new.set_critical_period(p.critical_period);
        return new;
    }
}

impl<'de> Deserialize<'de> for Params {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let res = _Params::deserialize(deserializer);
        return res.map(|x| Params::from(x));
    }
}

#[derive(CopyGetters, Setters, Debug, PartialEq, Copy, Clone, Default)]
#[getset(get = "pub", set = "pub")]
pub struct HealthcareCapacity {
    num_beds: usize,
    num_icus: usize,
    occupied_beds: usize,
    occupied_icus: usize,
    maximum_overflow_beds: usize,
    maximum_overflow_icus: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let params = Params::default();
        let data = toml::to_string(&params).unwrap();
        let params_: Params = toml::from_str(&data).unwrap();
        assert_eq!(params, params_);
    }
}
