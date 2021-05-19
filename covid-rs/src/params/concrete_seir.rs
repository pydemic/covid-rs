use super::{
    bound_params::{ForState, MapComponents},
    seir::{daily_probability, SEIRParams, SEIRParamsData},
    universal_seir::UniversalSEIRParams,
    BoundParams,
};
use crate::prelude::{AgeDistribution10, AgeParam, Real};
use getset::{Getters, Setters};
use paste::paste;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

///////////////////////////////////////////////////////////////////////////////
// Default param for COVID-19
///////////////////////////////////////////////////////////////////////////////

const PROB_ASYMPTOMATIC: Real = 0.42;
const PROB_SEVERE: Real = 0.18;
const PROB_CRITICAL: Real = 0.22;
const PROB_DEATH: Real = 0.42;
const CASE_FATALITY_RATIO: Real = PROB_SEVERE * PROB_CRITICAL * PROB_DEATH;
//const INFECTION_FATALITY_RATIO: Real = CASE_FATALITY_RATIO * PROB_ASYMPTOMATIC;
const ASYMPTOMATIC_INFECTIOUSNESS: Real = 0.50;
const INCUBATION_PERIOD: Real = 3.69;
const INFECTIOUS_PERIOD: Real = 3.47;
const SEVERE_PERIOD: Real = 7.19;
const CRITICAL_PERIOD: Real = 17.50 - 7.19;

/// Trait for types that can be created from a UniversalSEIRParams implementation
pub trait FromUniversalParams {
    /// Create new instances from an UniversalSEIRParams implementation
    fn from_universal_params(params: &impl UniversalSEIRParams) -> Self;
}

///////////////////////////////////////////////////////////////////////////////
// Auxiliary macros
///////////////////////////////////////////////////////////////////////////////

/// Create a method for SEIRParams or SEIRParamsData traits
macro_rules! expand_methods {
    // Create functions that receive no arguments but self
    (
        $(by_field: { $($name:ident),* $(,)? })?
        $(by_value: { $($vname:ident: $value:expr),* $(,)? })?
        $(delegate[$delegate:ident]: { $($dname:ident),* $(,)? })?
        $(forward[$forward:ident]: { $($fname:ident),* $(,)? })?
    ) => {
        $($(
            fn $name(&self) -> Real {
                self.$name
            }
        )*)*
        $($(
            fn $vname(&self) -> Real {
                $value
            }
        )*)*
        $($(
            paste! {
                fn $dname(&self) -> Real {
                    self.$delegate.$dname()
                }
            }
        )*)*
        $($(
            paste! {
                fn $fname(&self) -> Real {
                    self.$forward.$fname
                }
            }
        )*)*
    };

    // Create functions that bind to an argument of type $ty
    (
        $(by_field[$ty:ident]: { $($name:ident),* $(,)? })?
        $(by_value[$vty:ident]: { $($vname:ident: $value:expr),* $(,)? })?
    ) => {
        $($(
            fn $name(&self, obj: &$ty) -> Real {
                self.$name.for_state(obj)
            }
        )*)*
        $($(
            fn $vname(&self, _: &$vty) -> Real {
                $value
            }
        )*)*
    };
}

/// Create a method for SEIRParams or SEIRParamsData traits
macro_rules! method {
    ($name:ident[$ty:ty], delegate=$delegate:ident) => {
        paste! {
            fn $name(&self, obj: &$ty) -> Real {
                self.$delegate.$name(obj)
            }
        }
    };
    ($name:ident<$ty:ident>) => {
        pub fn $name<$ty>(&self, obj: &$ty) -> Real
        where
            T: ForState<$ty, Output = Real>,
        {
            self.$name.for_state(obj)
        }
    };
    (data = $name:ident[$ty:ty]) => {
        paste! {
            fn [<with_ $name _data>]<S>(&self, f: impl FnOnce(&$ty) -> S) -> S {
                f(&self.$name)
            }
        }
    };
    (data = $name:ident[$ty:ty], value = $value:expr) => {
        paste! {
            fn [<with_ $name _data>]<S>(&self, f: impl FnOnce(&$ty) -> S) -> S
            {
                self.with_scalar_data($value, f)
            }
        }
    };
    (data = $name:ident[$ty:ty], delegate=$delegate:ident) => {
        paste! {
            fn [<with_ $name _data>]<S>(&self, f: impl FnOnce(&$ty) -> S) -> S {
                f(&self.$delegate.$name)
            }
        }
    };
}

////////////////////////////////////////////////////////////////////////////////
// Declare structs and trait implementations
////////////////////////////////////////////////////////////////////////////////

// EpidemicSEIRParams //////////////////////////////////////////////////////////

/// EpidemicSEIRParams store the basic epidemiological params. It can implement
/// SEIRParams trait assuming the default clinical evolution I -> H -> C -> {R, D}.
///
/// Compose with ClinicalSEIRParams to get the full set or compose with
/// CachedSEIRParams to obtain a cached version of those parameters.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct EpidemicSEIRParams<T> {
    incubation_period: T,
    infectious_period: T,
    asymptomatic_infectiousness: T,
    prob_asymptomatic: T,
    case_fatality_ratio: T,
}

impl<T: Default> Default for EpidemicSEIRParams<T> {
    default fn default() -> Self {
        EpidemicSEIRParams {
            incubation_period: T::default(),
            infectious_period: T::default(),
            asymptomatic_infectiousness: T::default(),
            prob_asymptomatic: T::default(),
            case_fatality_ratio: T::default(),
        }
    }
}

impl<T> EpidemicSEIRParams<T> {
    pub fn new(
        incubation_period: T,
        infectious_period: T,
        asymptomatic_infectiousness: T,
        prob_asymptomatic: T,
        case_fatality_ratio: T,
    ) -> Self {
        EpidemicSEIRParams {
            incubation_period,
            infectious_period,
            asymptomatic_infectiousness,
            prob_asymptomatic,
            case_fatality_ratio,
        }
    }

    fn default_components() -> Self
    where
        T: MapComponents<Elem = Real>,
    {
        EpidemicSEIRParams {
            incubation_period: T::from_component(INCUBATION_PERIOD),
            infectious_period: T::from_component(INFECTIOUS_PERIOD),
            asymptomatic_infectiousness: T::from_component(ASYMPTOMATIC_INFECTIOUSNESS),
            prob_asymptomatic: T::from_component(PROB_ASYMPTOMATIC),
            case_fatality_ratio: T::from_component(CASE_FATALITY_RATIO),
        }
    }

    /// Maps each param to function and construct a new EpidemicSEIRParams
    pub fn map<S>(&self, f: impl Fn(&T) -> S) -> EpidemicSEIRParams<S> {
        EpidemicSEIRParams {
            incubation_period: f(&self.incubation_period),
            infectious_period: f(&self.infectious_period),
            asymptomatic_infectiousness: f(&self.asymptomatic_infectiousness),
            prob_asymptomatic: f(&self.prob_asymptomatic),
            case_fatality_ratio: f(&self.case_fatality_ratio),
        }
    }
}

impl<T, S> SEIRParams<S> for EpidemicSEIRParams<T>
where
    T: ForState<S, Output = Real>,
{
    expand_methods!(
        by_field[S]: {
            incubation_period,
            infectious_period,
            asymptomatic_infectiousness,
            prob_asymptomatic,
            case_fatality_ratio,
        }
        by_value[S]: {
            severe_period: 0.0,
            critical_period: 0.0,
            prob_severe: 1.0,
            prob_critical: 1.0,
        }
    );
}

impl UniversalSEIRParams for EpidemicSEIRParams<Real> {
    expand_methods!(
        by_field: {
            incubation_period,
            infectious_period,
            asymptomatic_infectiousness,
            prob_asymptomatic,
            case_fatality_ratio,
        }
        by_value: {
            severe_period: 0.0,
            critical_period: 0.0,
            prob_critical: 1.0,
            prob_severe: 1.0,
        }
    );
}

impl<T> SEIRParamsData<T> for EpidemicSEIRParams<T>
where
    T: MapComponents<Elem = Real>,
{
    method!(data = incubation_period[T]);
    method!(data = infectious_period[T]);
    method!(data = severe_period[T], value = 0.0);
    method!(data = critical_period[T], value = 0.0);
}

impl FromUniversalParams for EpidemicSEIRParams<Real> {
    fn from_universal_params(params: &impl UniversalSEIRParams) -> Self {
        Self::new(
            params.incubation_period(),
            params.infectious_period(),
            params.asymptomatic_infectiousness(),
            params.prob_asymptomatic(),
            params.case_fatality_ratio(),
        )
    }
}

// ClinicalSEIRParams //////////////////////////////////////////////////////////

/// ClinicalSEIRParams store information about the clinical evolution of cases.
///
/// Composing with EpidemicSEIRParams, it is possible to implement arbitrary values
/// for the full set of SEIRParams methods.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ClinicalSEIRParams<T> {
    severe_period: T,
    critical_period: T,
    prob_severe: T,
    prob_critical: T,
}

impl<T> ClinicalSEIRParams<T> {
    pub fn new(severe_period: T, critical_period: T, prob_severe: T, prob_critical: T) -> Self {
        ClinicalSEIRParams {
            severe_period,
            critical_period,
            prob_severe,
            prob_critical,
        }
    }

    fn default_components() -> Self
    where
        T: MapComponents<Elem = Real>,
    {
        ClinicalSEIRParams {
            severe_period: T::from_component(SEVERE_PERIOD),
            critical_period: T::from_component(CRITICAL_PERIOD),
            prob_severe: T::from_component(PROB_SEVERE),
            prob_critical: T::from_component(PROB_CRITICAL),
        }
    }

    /// Maps function to each component of struct
    pub fn map<S>(&self, f: impl Fn(&T) -> S) -> ClinicalSEIRParams<S> {
        ClinicalSEIRParams {
            severe_period: f(&self.severe_period),
            critical_period: f(&self.critical_period),
            prob_severe: f(&self.prob_severe),
            prob_critical: f(&self.prob_critical),
        }
    }

    pub fn severe_transition_prob<S>(&self, obj: &S) -> Real
    where
        T: ForState<S, Output = Real>,
    {
        daily_probability(self.severe_period(obj))
    }

    pub fn critical_transition_prob<S>(&self, obj: &S) -> Real
    where
        T: ForState<S, Output = Real>,
    {
        daily_probability(self.critical_period(obj))
    }

    method!(severe_period<S>);
    method!(critical_period<S>);
    method!(prob_severe<S>);
    method!(prob_critical<S>);
    // method!(data = severe_period[T]);
    // method!(data = critical_period[T]);
}

impl<T: Default> Default for ClinicalSEIRParams<T> {
    default fn default() -> Self {
        ClinicalSEIRParams {
            severe_period: T::default(),
            critical_period: T::default(),
            prob_severe: T::default(),
            prob_critical: T::default(),
        }
    }
}

impl FromUniversalParams for ClinicalSEIRParams<Real> {
    fn from_universal_params(params: &impl UniversalSEIRParams) -> Self {
        Self::new(
            params.severe_period(),
            params.critical_period(),
            params.prob_severe(),
            params.prob_critical(),
        )
    }
}

// FullSEIRParams //////////////////////////////////////////////////////////

/// Compose EpidemicSEIRParams with ClinicalSEIRParams.
#[derive(Copy, Clone, Debug, PartialEq, Getters, Setters, Serialize, Default)]
#[serde(default)]
#[getset(get = "pub", set = "pub")]
pub struct FullSEIRParams<T> {
    epidemic: EpidemicSEIRParams<T>,
    clinical: ClinicalSEIRParams<T>,
}

impl<T> FullSEIRParams<T> {
    pub fn new(epidemic: EpidemicSEIRParams<T>, clinical: ClinicalSEIRParams<T>) -> Self {
        FullSEIRParams { epidemic, clinical }
    }

    pub fn map<S>(&self, f: impl Fn(&T) -> S) -> FullSEIRParams<S> {
        FullSEIRParams::new(self.epidemic.map(&f), self.clinical.map(&f))
    }
}

impl<T, S> SEIRParams<S> for FullSEIRParams<T>
where
    T: MapComponents<Elem = Real> + ForState<S, Output = Real> + Default,
{
    // Epidemic methods
    method!(incubation_period[S], delegate = epidemic);
    method!(infectious_period[S], delegate = epidemic);
    method!(asymptomatic_infectiousness[S], delegate = epidemic);
    method!(prob_asymptomatic[S], delegate = epidemic);
    method!(case_fatality_ratio[S], delegate = epidemic);
    method!(incubation_transition_prob[S], delegate = epidemic);
    method!(infectious_transition_prob[S], delegate = epidemic);

    // Clinical methods
    method!(severe_period[S], delegate = clinical);
    method!(critical_period[S], delegate = clinical);
    method!(prob_severe[S], delegate = clinical);
    method!(prob_critical[S], delegate = clinical);
    method!(severe_transition_prob[S], delegate = clinical);
    method!(critical_transition_prob[S], delegate = clinical);
}

impl UniversalSEIRParams for FullSEIRParams<Real> {
    // Epidemic methods
    expand_methods!(
        delegate[epidemic]: {
            incubation_period,
            infectious_period,
            asymptomatic_infectiousness,
            prob_asymptomatic,
            case_fatality_ratio,
            incubation_transition_prob,
            infectious_transition_prob,
        }
        forward[clinical]: {
            severe_period,
            critical_period,
            prob_severe,
            prob_critical,
        }
    );
}

impl<T> SEIRParamsData<T> for FullSEIRParams<T>
where
    T: MapComponents<Elem = Real>,
{
    method!(data = incubation_period[T], delegate = epidemic);
    method!(data = infectious_period[T], delegate = epidemic);
    method!(data = severe_period[T], delegate = clinical);
    method!(data = critical_period[T], delegate = clinical);
}

impl FromUniversalParams for FullSEIRParams<Real> {
    fn from_universal_params(params: &impl UniversalSEIRParams) -> Self {
        Self::new(
            FromUniversalParams::from_universal_params(params),
            FromUniversalParams::from_universal_params(params),
        )
    }
}

// CachedSEIRParams ////////////////////////////////////////////////////////////

/// A cached params take a params impl and caches all transition probability
/// values. This avoids some potentially expensive computations involving
/// exponential by paying a fixed cost upfront when writing data for each
/// corresponding transition period.
#[derive(Copy, Clone, Debug, PartialEq, Getters)]
pub struct CachedSEIRParams<P, T> {
    #[getset(get = "pub")]
    params: P,
    incubation_transition_prob: T,
    infectious_transition_prob: T,
    severe_transition_prob: T,
    critical_transition_prob: T,
}

impl<P, T> CachedSEIRParams<P, T>
where
    P: SEIRParamsData<T>,
    T: MapComponents<Elem = Real>,
{
    pub fn new(params: P) -> Self {
        CachedSEIRParams {
            incubation_transition_prob: params
                .with_incubation_period_data(|xs| xs.map_components(daily_probability)),
            infectious_transition_prob: params
                .with_infectious_period_data(|xs| xs.map_components(daily_probability)),
            severe_transition_prob: params
                .with_severe_period_data(|xs| xs.map_components(daily_probability)),
            critical_transition_prob: params
                .with_critical_period_data(|xs| xs.map_components(daily_probability)),
            params,
        }
    }
}

impl<P, T> Default for CachedSEIRParams<P, T>
where
    P: SEIRParamsData<T> + Default,
    T: MapComponents<Elem = Real>,
{
    default fn default() -> Self {
        let params = P::default();
        Self::new(params)
    }
}

impl<P, T, S> SEIRParams<S> for CachedSEIRParams<P, T>
where
    P: SEIRParams<S>,
    T: ForState<S, Output = Real>,
{
    // Delegate to attributes
    method!(incubation_period[S], delegate = params);
    method!(infectious_period[S], delegate = params);
    method!(severe_period[S], delegate = params);
    method!(critical_period[S], delegate = params);
    method!(asymptomatic_infectiousness[S], delegate = params);
    method!(prob_asymptomatic[S], delegate = params);
    method!(prob_severe[S], delegate = params);
    method!(prob_critical[S], delegate = params);
    method!(prob_death[S], delegate = params);
    method!(case_fatality_ratio[S], delegate = params);
    method!(infection_fatality_ratio[S], delegate = params);

    // Read directly from attributes
    expand_methods!(
       by_field[S]: {
            incubation_transition_prob,
            infectious_transition_prob,
            severe_transition_prob,
            critical_transition_prob,
        }
    );
}

impl<P> UniversalSEIRParams for CachedSEIRParams<P, Real>
where
    P: UniversalSEIRParams,
{
    // Epidemic methods
    expand_methods!(
        delegate[params]: {
            // Epidemic
            incubation_period,
            infectious_period,
            asymptomatic_infectiousness,
            prob_asymptomatic,
            case_fatality_ratio,
            infection_fatality_ratio,
            prob_death,

            // Clinical
            severe_period,
            critical_period,
            prob_severe,
            prob_critical,
        }
    );

    // Read directly from attributes
    expand_methods!(
       by_field: {
            incubation_transition_prob,
            infectious_transition_prob,
            severe_transition_prob,
            critical_transition_prob,
        }
    );
}

impl<P> FromUniversalParams for CachedSEIRParams<P, Real>
where
    P: FromUniversalParams + SEIRParamsData<Real>,
{
    fn from_universal_params(params: &impl UniversalSEIRParams) -> Self {
        let src: P = FromUniversalParams::from_universal_params(params);
        Self::new(src)
    }
}

macro_rules! register_defaults {
    ($ty:ty) => {
        impl Default for EpidemicSEIRParams<$ty> {
            fn default() -> Self {
                Self::default_components()
            }
        }

        impl Default for ClinicalSEIRParams<$ty> {
            fn default() -> Self {
                Self::default_components()
            }
        }
    };
}

register_defaults!(Real);
register_defaults!(AgeDistribution10);
register_defaults!(AgeParam);
