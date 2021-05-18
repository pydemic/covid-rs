use crate::prelude::Real;

use super::{daily_probability, SEIRParams};

macro_rules! method {
    ($name:ident) => {
        fn $name(&self) -> Real;
    };

    ($name:ident => $expr:expr) => {
        fn $name(&self) -> Real {
            $expr
        }
    };
    ($id:ident(())) => {
        fn $id(&self) -> Real {
            <Self as SEIRParams<()>>::$id(self, &())
        }
    };
}

/// A trait that provide descriptions of epidemiological parameters independently
/// from any agent state. The API replicates most of SEIRParams methods without
/// requiring and have the same meaning.
pub trait UniversalSEIRParams {
    method!(incubation_period);
    method!(infectious_period);
    method!(severe_period);
    method!(critical_period);
    method!(asymptomatic_infectiousness);
    method!(prob_asymptomatic );
    method!(prob_severe);
    method!(prob_critical);
    method!(case_fatality_ratio);

    fn prob_death(&self) -> Real {
        let factor = self.prob_critical() * self.prob_severe();
        return self.case_fatality_ratio() / factor;
    }

    fn infection_fatality_ratio(&self) -> Real {
        self.case_fatality_ratio() * (1.0 - self.prob_asymptomatic())
    }

    fn incubation_transition_prob(&self) -> Real {
        self.daily_probability(self.incubation_period())
    }

    fn infectious_transition_prob(&self) -> Real {
        self.daily_probability(self.infectious_period())
    }

    fn severe_transition_prob(&self) -> Real {
        self.daily_probability(self.severe_period())
    }

    fn critical_transition_prob(&self) -> Real {
        self.daily_probability(self.severe_period())
    }

    /// A helper method that computes the daily transition probability from the
    /// transition period.
    #[inline]
    fn daily_probability(&self, value: Real) -> Real {
        daily_probability(value)
    }
}

////////////////////////////////////////////////////////////////////////////////
// Trait implementations
////////////////////////////////////////////////////////////////////////////////

impl<T> UniversalSEIRParams for T
where
    T: SEIRParams<()>,
{
    method!(incubation_period(()));
    method!(infectious_period(()));
    method!(severe_period(()));
    method!(critical_period(()));
    method!(asymptomatic_infectiousness(()));
    method!(prob_asymptomatic(()));
    method!(prob_severe(()));
    method!(prob_critical(()));
    method!(prob_death(()));
    method!(case_fatality_ratio(()));
    method!(infection_fatality_ratio(()));
    method!(incubation_transition_prob(()));
    method!(infectious_transition_prob(()));
    method!(severe_transition_prob(()));
    method!(critical_transition_prob(()));
}
