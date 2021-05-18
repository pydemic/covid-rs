use getset::{Getters, Setters};

use crate::{
    prelude::{Age, AgeParam, ForAge, Real},
    sim::HasAge,
};

use super::{SEIRParams, UniversalSEIRParams};

/// A trait that maps Self with the expected output of a ParamSet after receiving
/// some state S as argument.
///
/// A simple example: Self might be an array of reals representing a probability
/// distribution, S can be an age and Output is the value corresponding to each
/// age group. The for_state() method maps ages to those values using the
/// internal array data from Self.
pub trait ForState<S> {
    type Output;

    /// Maps a value of obj to the desired Output value.
    fn for_state(&self, obj: &S) -> Self::Output;
}

/// A trait related to ForState, which allows transformation of the inner data
/// by some mapping in the expected elements.
///
/// This trait is natural for types that somehow store a collection of elements
/// that can be retrieved by some key S using the ForState<S, Output=Elem> trait.
pub trait MapComponents
where
    Self: Sized,
{
    type Elem;

    /// Uses f(x) to transform self internally and return a mapped result.
    fn map_components(&self, f: impl Fn(Self::Elem) -> Self::Elem) -> Self;

    /// Create data from single element, possibly replicating it for all keys.
    fn from_component(x: Self::Elem) -> Self;
}

/// A parameter set bound to a given state.
///
/// Impls should provide an API that mimics the wrapped Param Set omitting the
/// state argument which is passed implicitly from the state stored in this
/// struct.
#[derive(Debug, Clone, Copy, Getters, Setters, Default, PartialEq)]
#[getset(get = "pub", set = "pub")]
pub struct BoundParams<P, S> {
    params: P,
    bind: S,
}

impl<P, S> BoundParams<P, S> {
    /// Binds a param set to state
    pub fn new(params: P, bind: S) -> Self {
        BoundParams { params, bind }
    }

    /// Return mutable reference to params.
    pub fn params_mut(&mut self) -> &mut P {
        &mut self.params
    }
}

/// A parameter set bound to a given state.
///
/// This is similar to BoundParams, but uses references instead of owning the
/// param set and state. Using this struct is potentially much more efficient
/// since it avoids unnecessary copies, but may lead to some lifetime issues
/// in some situations.
#[derive(Debug)]
pub struct BoundParamsRef<'a, P, B> {
    params: &'a P,
    bind: B,
}

impl<'a, P, S> BoundParamsRef<'a, P, S> {
    /// Binds a reference to a param set to state
    pub fn new(params: &'a P, bind: S) -> Self {
        BoundParamsRef { params, bind }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Trait implementations
////////////////////////////////////////////////////////////////////////////////

impl<T, S> ForState<S> for T
where
    T: ForAge,
    S: HasAge,
{
    type Output = T::Output;

    #[inline]
    default fn for_state(&self, obj: &S) -> T::Output {
        self.for_age(obj.age())
    }
}

impl<T: ForAge<Output = Real>> ForState<Age> for T {
    type Output = Real;

    #[inline]
    default fn for_state(&self, age: &Age) -> Real {
        self.for_age(*age)
    }
}

impl MapComponents for Real {
    type Elem = Real;

    fn map_components(&self, f: impl Fn(Self::Elem) -> Self::Elem) -> Self {
        f(*self)
    }

    fn from_component(x: Self::Elem) -> Self {
        x
    }
}

impl<T, const N: usize> MapComponents for [T; N]
where
    T: Sized + Copy,
{
    type Elem = T;

    fn map_components(&self, f: impl Fn(Self::Elem) -> Self::Elem) -> Self {
        (&self).map(f)
    }

    fn from_component(x: Self::Elem) -> Self {
        [x; N]
    }
}

impl MapComponents for AgeParam {
    type Elem = Real;

    fn map_components(&self, f: impl Fn(Self::Elem) -> Self::Elem) -> Self {
        self.map(f)
    }

    fn from_component(x: Self::Elem) -> Self {
        AgeParam::Scalar(x)
    }
}

/// Implement boilerplate for bound SEIRParam methods
macro_rules! bound_method {
    ($id:ident) => {
        fn $id(&self) -> Real {
            self.params.$id(&self.bind)
        }
    };
    (& $id:ident) => {
        fn $id(&self) -> Real {
            self.params.$id(&self.bind)
        }
    };
}

impl<P: SEIRParams<S>, S> UniversalSEIRParams for BoundParams<P, S> {
    bound_method!(incubation_period);
    bound_method!(infectious_period);
    bound_method!(severe_period);
    bound_method!(critical_period);
    bound_method!(asymptomatic_infectiousness);
    bound_method!(prob_asymptomatic);
    bound_method!(prob_severe);
    bound_method!(prob_critical);
    bound_method!(prob_death);
    bound_method!(case_fatality_ratio);
    bound_method!(infection_fatality_ratio);
    bound_method!(incubation_transition_prob);
    bound_method!(infectious_transition_prob);
    bound_method!(severe_transition_prob);
    bound_method!(critical_transition_prob);
}

impl<'a, P: SEIRParams<S>, S> UniversalSEIRParams for BoundParamsRef<'a, P, S> {
    bound_method!(&incubation_period);
    bound_method!(&infectious_period);
    bound_method!(&severe_period);
    bound_method!(&critical_period);
    bound_method!(&asymptomatic_infectiousness);
    bound_method!(&prob_asymptomatic);
    bound_method!(&prob_severe);
    bound_method!(&prob_critical);
    bound_method!(&prob_death);
    bound_method!(&case_fatality_ratio);
    bound_method!(&infection_fatality_ratio);
    bound_method!(&incubation_transition_prob);
    bound_method!(&infectious_transition_prob);
    bound_method!(&severe_transition_prob);
    bound_method!(&critical_transition_prob);
}
