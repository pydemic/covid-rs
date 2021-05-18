use super::{
    AgeDependentSEIR, MapComponents, SEIRParamSet, SEIRParams, SEIRUniversalParamSet,
    UniversalSEIRParams,
};
use crate::{
    models::SimpleAgent,
    prelude::{Age, Real},
    sim::HasAge,
};
use getset::*;

/// This trait binds global parameters to a local view seen by each agent.
///
/// This view can depend on the agent's state or (more typically) on parts of
/// the agent's state. A typical example is a set of parameters that depend on
/// age can be bound to some agent's age before passing to its update functions.
/// The bound parameters would only show values pertinent to the agent's specific
/// age.
///
/// The local bind must also expose a mutable set of world parameters. Those
/// parameters are exposed as references and thus must be owned by self.
pub trait LocalBind<S> {
    type Local;
    type World;
    type Bind;

    /// Bind parameters to value
    fn bind(&mut self, bind: Self::Bind);

    /// Return local parameters for current bind
    fn local(&self) -> &Self::Local;

    /// Return a reference to the world parameters
    fn world(&self) -> &Self::World;

    /// Return a mutable reference to the world parameters
    fn world_mut(&mut self) -> &mut Self::World;

    /// Just a convenience function that extract bind data and then binds
    fn bind_to_object(&mut self, _: &S);
}

impl<S> LocalBind<S> for SEIRUniversalParamSet {
    type Local = Self;
    type World = Self;
    type Bind = S;

    fn bind(&mut self, _: Self::Bind) {}

    fn bind_to_object(&mut self, _: &S) {}

    fn local(&self) -> &Self::Local {
        self
    }

    fn world(&self) -> &Self::World {
        self
    }

    fn world_mut(&mut self) -> &mut Self::World {
        self
    }
}

impl<T, D> LocalBind<T> for AgeDependentSEIR<D>
where
    T: HasAge,
    D: MapComponents<Elem = Real>,
{
    type Local = Self;
    type World = SEIRParamSet<D>;
    type Bind = Age;

    default fn bind(&mut self, bind: Age) {
        self.set_bind(bind);
    }

    default fn local(&self) -> &Self::Local {
        self
    }

    default fn bind_to_object(&mut self, obj: &T) {
        let age = obj.age();
        <AgeDependentSEIR<D> as LocalBind<T>>::bind(self, age);
    }

    default fn world(&self) -> &Self::World {
        self.params()
    }

    default fn world_mut(&mut self) -> &mut Self::World {
        self.params_mut()
    }
}

////////////////////////////////////////////////////////////////////////////////
// Bound by age and vaccine
////////////////////////////////////////////////////////////////////////////////

macro_rules! methods {
    (unaffected: { $($name:ident),* $(,)? }) => {
        $(
            fn $name(&self) -> Real {
                self.params.$name(&self.age)
            }
        )*
    };
    (efficient: { $($name:ident),* $(,)? }) => {
        $(
            fn $name(&self) -> Real {
                if self.vaccine {
                    return 0.0;
                }
                self.params.$name(&self.age)
            }
        )*
    };
}

/// This simple struct binds a group of parameters by age and vaccine. We assume
/// that parameters depend only on age, and vaccine affect other probabilities
/// and parameters in an age-dependent universal way.
#[derive(Debug, Clone, Copy, Getters, Setters, Default, PartialEq)]
#[getset(get = "pub", set = "pub")]
pub struct SimpleVaccineBound<P> {
    params: P,
    age: Age,
    vaccine: bool,
}

impl<D> LocalBind<SimpleAgent<(), bool>> for SimpleVaccineBound<SEIRParamSet<D>>
where
    D: MapComponents<Elem = Real>,
{
    type Local = Self;
    type World = SEIRParamSet<D>;
    type Bind = (Age, bool);

    fn bind(&mut self, bind: (Age, bool)) {
        self.age = bind.0;
        self.vaccine = bind.1;
    }

    fn local(&self) -> &Self::Local {
        self
    }

    fn world(&self) -> &Self::World {
        self.params()
    }

    fn world_mut(&mut self) -> &mut Self::World {
        &mut self.params
    }

    fn bind_to_object(&mut self, obj: &SimpleAgent<(), bool>) {
        let age = obj.age();
        let vaccine = obj.vaccine().clone();
        let bind = (age, vaccine);
        <SimpleVaccineBound<SEIRParamSet<D>> as LocalBind<SimpleAgent<(), bool>>>::bind(self, bind);
    }
}

impl<P> UniversalSEIRParams for SimpleVaccineBound<P>
where
    P: SEIRParams<Age>,
{
    methods!(
        unaffected: {
            incubation_period,
            infectious_period,
            severe_period,
            critical_period,

            incubation_transition_prob,
            infectious_transition_prob,
            severe_transition_prob,
            critical_transition_prob,

            asymptomatic_infectiousness,
            prob_asymptomatic,
        }
    );

    methods!(
        efficient: {
            prob_severe,
            prob_critical,
            case_fatality_ratio,
        }
    );
}
