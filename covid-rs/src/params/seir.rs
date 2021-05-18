use super::{BoundParams, BoundParamsRef, MapComponents};
use crate::prelude::Real;

/// on some state value. If no such dependency exists, the trait can be
/// specialized to SEIRParams<()>.
///
/// State must be feed as an additional argument to the getter functions of
/// this trait.
pub trait SEIRParams<S> {
    /// Incubation period is the average duration in the "Exposed" category.
    /// Agents are infected but *CANNOT* infect other agents.
    fn incubation_period(&self, obj: &S) -> Real;

    /// Incubation period is the average duration in the "Infectious" category"
    /// Agents are infected and *CAN* infect other agents.
    fn infectious_period(&self, obj: &S) -> Real;

    /// Average duration of a "severe" case. 
    ///
    /// A value of zero  is equivalent to disabling the clinical evolution in the
    /// CH compartments, effectively transforming SEICHAR to SEIR.
    fn severe_period(&self, _obj: &S) -> Real;

    /// Average duration of a "critical" case. 
    ///
    /// Like severe_period() a null value makes it coincide with SEIR;
    fn critical_period(&self, _obj: &S) -> Real;

    /// Relative infectiousness of asymptomatic agents.
    ///
    /// Make it equal to 1.0 to coincide with SEIR;
    fn asymptomatic_infectiousness(&self, _obj: &S) -> Real;

    /// Probability of an exposed agent not developing any symptoms (E to A).
    ///
    /// The complement is the probability for transitioning from E to I.
    ///
    /// A value of 0.0 makes it coincide with SEIR;
    fn prob_asymptomatic(&self, _obj: &S) -> Real;

    /// Probability of an infectious agent develop severe symptoms (I to H).
    ///
    /// The complement is the probability for transitioning from I to R.
    ///
    /// A value of 1.0 makes it coincide with SEIR and keep the fatality rate.
    /// A value of 0.0 imposes a transition to R, with zero chance of deaths.
    fn prob_severe(&self, _obj: &S) -> Real;

    /// Probability of a severe agent develop critical symptoms (S to C).
    ///
    /// The complement is the probability for transitioning from S to R.
    ///
    /// Must adopt the same values of prob_severe to coincide with SEIR.
    fn prob_critical(&self, _obj: &S) -> Real;

    /// Probability of a severe critical agent to die (C to D).
    ///
    /// The complement is the probability for transitioning from C to R.
    /// The default value uses CFR and the transition probabilities I -> S and
    /// S -> I
    fn prob_death(&self, obj: &S) -> Real {
        let factor = self.prob_critical(obj) * self.prob_severe(obj);
        return self.case_fatality_ratio(obj) / factor;
    }

    /// Probability of death for (symptomatic) cases.
    /// Defaults to zero.
    fn case_fatality_ratio(&self, _obj: &S) -> Real;

    /// Probability of death for all infections (symptomatic or not)
    ///
    /// Impls should usually override case_fatality_ratio() and prob_asymptomatic()
    /// and use the default implementation of this method.
    fn infection_fatality_ratio(&self, obj: &S) -> Real {
        self.case_fatality_ratio(obj) * (1.0 - self.prob_asymptomatic(obj))
    }

    /// Probability of transition E -> (A or I) in a single day.
    fn incubation_transition_prob(&self, obj: &S) -> Real {
        self.daily_probability(self.incubation_period(obj))
    }

    /// Probability of transition I -> (H or R) in a single day.
    fn infectious_transition_prob(&self, obj: &S) -> Real {
        self.daily_probability(self.infectious_period(obj))
    }

    /// Probability of transition S -> (C or R) in a single day.
    fn severe_transition_prob(&self, obj: &S) -> Real {
        self.daily_probability(self.severe_period(obj))
    }

    /// Probability of transition C -> (D or R) in a single day.
    fn critical_transition_prob(&self, obj: &S) -> Real {
        self.daily_probability(self.severe_period(obj))
    }

    /// A helper method that computes the daily transition probability from the
    /// transition period.
    #[inline]
    fn daily_probability(&self, value: Real) -> Real {
        daily_probability(value)
    }

    /// Creates a bound SimpleSEIRParams object bound to the given state.
    ///
    /// It binds to a reference to self, which is more efficient, but may
    /// create problems with lifetimes.
    fn bind<'a>(&'a self, bind: S) -> BoundParamsRef<'a, Self, S>
    where
        Self: Sized,
        S: Clone,
    {
        BoundParamsRef::new(self, bind)
    }

    /// Creates a bound SimpleSEIRParams object bound to the given state.
    ///
    fn bind_copy(&self, obj: &S) -> BoundParams<Self, S>
    where
        Self: Clone,
        S: Clone,
    {
        BoundParams::new(self.clone(), obj.clone())
    }

    /// Creates a bound SimpleSEIRParams object bound to the given state.
    ///
    /// It binds to a reference to self, which is more efficient, but may
    /// create problems with lifetimes.
    fn with_bounded_params<R>(
        &self,
        bind: S,
        f: impl FnOnce(&BoundParamsRef<'_, Self, S>) -> R,
    ) -> R
    where
        Self: Sized,
    {
        // let ptr: *mut S = obj;
        let params = BoundParamsRef::new(self, bind);
        f(&params)
        // // Safety: object cannot be moved
        // unsafe {
        //     f(&params, &mut *ptr);
        // }
        // drop(obj);
    }
}

/// A trait for objects tha expose the internal data representation T of the
/// SEIR param set.
pub trait SEIRParamsData<T> {
    fn with_incubation_period_data<S>(&self, f: impl FnOnce(&T) -> S) -> S;
    fn with_infectious_period_data<S>(&self, f: impl FnOnce(&T) -> S) -> S;
    fn with_severe_period_data<S>(&self, f: impl FnOnce(&T) -> S) -> S;
    fn with_critical_period_data<S>(&self, f: impl FnOnce(&T) -> S) -> S;

    /// Helper method that may make it easier to implement with_*_data() methods
    /// for missing values.
    fn with_scalar_data<R, S>(&self, scalar: R, f: impl FnOnce(&T) -> S) -> S
    where
        T: MapComponents<Elem = R>,
    {
        let data = T::from_component(scalar);
        f(&data)
    }
}

/// Computes the daily transition probability from the transition period.
#[inline(always)]
pub(crate) fn daily_probability(value: Real) -> Real {
    1.0 - (-1. / value).exp()
}
