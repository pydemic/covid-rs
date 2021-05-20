use super::{EpiParamsLocalT, EpiParamsT, EpiParamsGlobal, LocalBind, MapComponents};
use crate::{
    models::SimpleAgent,
    prelude::{Age, Real},
    sim::HasAge,
};
use getset::*;

/// This simple struct binds a group of parameters by age and vaccine. We assume
/// that parameters depend only on age, and vaccine affect other probabilities
/// and parameters in an age-dependent universal way.
#[derive(Debug, Clone, Copy, Getters, Setters, Default, PartialEq)]
#[getset(get = "pub", set = "pub")]
pub struct BindVaccine<P> {
    params: P,
    age: Age,
    vaccine: bool,
}

impl<M, D> LocalBind<SimpleAgent<M, bool>> for BindVaccine<EpiParamsGlobal<D>>
where
    D: MapComponents<Elem = Real> + Default,
{
    type Local = Self;
    type World = EpiParamsGlobal<D>;
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

    fn bind_to_object(&mut self, obj: &SimpleAgent<M, bool>) {
        let age = obj.age();
        let vaccine = obj.vaccine().clone();
        let bind = (age, vaccine);
        <BindVaccine<EpiParamsGlobal<D>> as LocalBind<SimpleAgent<(), bool>>>::bind(self, bind);
    }
}

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

impl<P> EpiParamsLocalT for BindVaccine<P>
where
    P: EpiParamsT<Age>,
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

impl<P> From<P> for BindVaccine<P> {
    fn from(params: P) -> Self {
        BindVaccine {
            params,
            age: 0,
            vaccine: false,
        }
    }
}
