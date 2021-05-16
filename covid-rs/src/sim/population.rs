use super::{Agent, Id, State, StochasticUpdate, Update, World};
use crate::prelude::{EpiModel, Sampler, SEAIRLike, SEICHARLike, SEIRLike, SIRLike};
use paste::paste;
use rand::prelude::{Rng, SliceRandom};
use std::collections::HashSet;

#[macro_export]
macro_rules! compartment_methods {
    ($id:ident, for=$typ:ty) => {
        paste! {
            fn [<each_ $id>](&self, f: impl FnMut(Id, &Self::State))
            where
                Self::State: $typ,
            {
                let mut g = f;
                self.each_agent(&mut |id, st: &Self::State| {
                    if st.[<is_ $id>]() {
                        g(id, st)
                    }
                })
            }

            fn [<each_ $id _mut>](&mut self, f: impl FnMut(Id, &mut Self::State))
            where
                Self::State: $typ,
            {
                let mut g = f;
                self.each_agent_mut(&mut |id, st: &mut Self::State| {
                    if st.[<is_ $id>]() {
                        g(id, st)
                    }
                })
            }

            fn [<n_ $id>](&self) -> usize
            where
                Self::State: $typ,
            {
                let mut n = 0;
                self.each_agent(&mut |_, _| n += 1);
                return n;
            }
        }
    };
}

/// The population trait describes a collection of agents.
pub trait Population {
    type State: State;

    fn from_states<I>(states: I) -> Self
    where
        I: IntoIterator<Item = Self::State>;

    fn to_states(&self) -> Vec<Self::State> {
        let mut vec = vec![];
        self.each_agent(&mut |_, a: &Self::State| vec.push(a.clone()));
        return vec;
    }

    /// Count the population size.
    fn count(&self) -> usize;

    /// Get an agent by id.
    fn get_agent(&self, id: Id) -> Option<Agent<Self::State>>;

    /// Get agents by ids. If you need to fetch multiple agents, it can be more
    /// convenient to use this.
    fn get_agents(&self, ids: impl IntoIterator<Item = Id>) -> Vec<Agent<Self::State>> {
        let mut out: Vec<Agent<Self::State>> = Vec::new();
        for id in ids.into_iter() {
            self.get_agent(id).map(|a| out.push(a.clone()));
        }
        return out;
    }

    /// Set an agent state by id.
    fn set_agent(&mut self, id: Id, state: &Self::State) -> &mut Self;

    /// Map function to agent, mutating it.
    fn map_agent_mut<B>(&mut self, id: Id, f: impl FnOnce(&mut Self::State) -> B) -> Option<B>;

    /// Map function to agent.
    fn map_agent<B>(&self, id: Id, f: impl FnOnce(&Self::State) -> B) -> Option<B>;

    /// Set multiple agent states by ids. If you need to update multiple
    /// agents, it can be more convenient to use this.
    fn set_agents(&mut self, updates: &[(Id, &Self::State)]) -> &mut Self {
        for &(id, state) in updates {
            self.set_agent(id, state);
        }
        return self;
    }

    /// Apply function to all agents of population. Function receives the Id
    /// and reference to State.
    fn each_agent<F>(&self, f: &mut F)
    where
        F: FnMut(Id, &Self::State);

    /// Apply function to all agents of population. Function receives the Id
    /// and mutable reference to State.
    fn each_agent_mut(&mut self, f: impl FnMut(Id, &mut Self::State));

    /// Select a random agent using random number generator.
    fn random<R: Rng>(&self, rng: &mut R) -> (Id, Self::State);

    /// Select many distinct random agents.
    fn randoms<R: Rng>(&self, count: usize, rng: &mut R) -> Vec<(Id, Self::State)> {
        self.randoms_with(count, rng, |st| st.clone())
    }

    /// Select many distinct random agents.
    fn randoms_with<T, R, F>(&self, count: usize, rng: &mut R, f: F) -> Vec<(Id, T)>
    where
        R: Rng,
        F: Fn(&Self::State) -> T,
    {
        let mut ids = HashSet::new();
        let mut out = Vec::new();

        while out.len() < count {
            let (id, state) = self.random(rng);
            if !ids.contains(&id) {
                ids.insert(id);
                out.push((id, f(&state)));
            }
        }

        return out;
    }

    fn update_random<W, R>(&mut self, world: &W, rng: &mut R)
    where
        R: Rng,
        W: World,
        <Self as Population>::State: StochasticUpdate<W>,
    {
        self.each_agent_mut(&mut |_, st: &mut Self::State| st.update_random(world, rng));
    }

    fn update_sampler<S, R>(&mut self, sampler: &S, rng: &mut R) -> usize
    where
        R: Rng,
        S: Sampler<Self>,
        Self: Sized,
        Self::State: SIRLike,
    {
        self.update_sampler_with(sampler, rng, |src, dest| dest.contaminated_from(src))
    }

    fn update_sampler_with<S, R>(
        &mut self,
        sampler: &S,
        rng: &mut R,
        f: impl FnMut(&Self::State, &Self::State) -> Option<Self::State>,
    ) -> usize
    where
        R: Rng,
        S: Sampler<Self>,
        Self: Sized,
        Self::State: SIRLike,
    {
        let mut cases = 0;
        let mut g = f;

        for (i, j) in sampler.sample_infection_pairs(self, rng) {
            if let (Some(src), Some(dest)) = (self.get_agent(i), self.get_agent(j)) {
                if let Some(contaminated) = g(&src.state, &dest.state) {
                    self.set_agent(j, &contaminated);
                    cases += 1;
                }
            }
            cases += 1;
        }
        return cases;
    }

    fn update<W>(&mut self, world: &W)
    where
        W: World,
        <Self as Population>::State: Update<W>,
    {
        self.each_agent_mut(|_, st: &mut Self::State| st.update(world));
    }

    fn contaminate_at_random(
        &mut self,
        n: usize,
        rng: &mut impl Rng,
        f: impl Fn(usize, &mut Self::State) -> bool,
    ) -> usize
    where
        Self::State: SIRLike,
    {
        let from_list = |n, rng, pop: &mut Self| {
            let mut susceptibles = vec![];
            pop.each_agent(&mut |id, st| {
                if st.is_susceptible() {
                    susceptibles.push(id);
                }
            });
            (susceptibles.len() > n).then(|| susceptibles.shuffle(rng));

            let mut size = susceptibles.len().min(n);
            for (_, id) in (0..size).into_iter().zip(susceptibles) {
                pop.map_agent_mut(id, |st| {
                    if f(id, st) {
                        size -= 1;
                    }
                });
            }
            return size;
        };

        let mut cases = 0;
        let mut tries = 0;

        while cases < n {
            let (id, _) = self.random(rng);
            self.map_agent_mut(id, |st| {
                if st.is_susceptible() && f(id, st) {
                    cases += 1;
                }
            });

            tries += 1;
            if tries >= 3 * n && tries > 15 {
                return cases + from_list(n - cases, rng, self);
            }
        }
        return n;
    }

    // Methods for SIR-based populations //////////////////////////////////////
    compartment_methods!(susceptible, for=EpiModel);
    compartment_methods!(contagious, for=SIRLike);
    compartment_methods!(dead, for=EpiModel);
    compartment_methods!(infectious, for=SIRLike);
    compartment_methods!(recovered, for=SIRLike);
    compartment_methods!(exposed, for=SEIRLike);
    compartment_methods!(asymptomatic, for=SEAIRLike);
    compartment_methods!(severe, for=SEICHARLike);
    compartment_methods!(critical, for=SEICHARLike);

    fn count_sir(&self) -> (usize, usize, usize, usize)
    where
        Self::State: SIRLike + EpiModel,
    {
        let (mut s, mut i, mut r, mut n) = (0, 0, 0, 0);
        self.each_agent(&mut |_, st: &Self::State| {
            if st.is_susceptible() {
                s += 1;
            } else if st.is_infectious() {
                i += 1;
            } else if st.is_recovered() {
                r += 1;
            }
            n += 1;
        });
        return (s, i, r, n);
    }

    fn count_seir(&self) -> (usize, usize, usize, usize, usize)
    where
        Self::State: SEIRLike,
    {
        let (mut s, mut e, mut i, mut r, mut n) = (0, 0, 0, 0, 0);
        self.each_agent(&mut |_, st: &Self::State| {
            if st.is_susceptible() {
                s += 1;
            } else if st.is_exposed() {
                e += 1;
            } else if st.is_infectious() {
                i += 1;
            } else if st.is_recovered() {
                r += 1;
            }
            n += 1;
        });
        return (s, e, i, r, n);
    }

    fn count_seair(&self) -> (usize, usize, usize, usize, usize, usize)
    where
        Self::State: SEAIRLike,
    {
        let (mut s, mut e, mut a, mut i, mut r, mut n) = (0, 0, 0, 0, 0, 0);
        self.each_agent(&mut |_, st: &Self::State| {
            if st.is_susceptible() {
                s += 1;
            } else if st.is_exposed() {
                e += 1;
            } else if st.is_asymptomatic() {
                a += 1;
            } else if st.is_infectious() {
                i += 1;
            } else if st.is_recovered() {
                r += 1;
            }
            n += 1;
        });
        return (s, e, a, i, r, n);
    }
}

/// Simple trait for implementations that store states in a Vec of states.
///
/// It automatically provides Population implementations for instances of this
/// trait.
pub trait OwnsStateSlice {
    type State: State;

    fn owned_data_from_states<I>(states: I) -> Self
    where
        I: IntoIterator<Item = Self::State>;

    fn as_state_slice(&self) -> &[Self::State];
    fn as_state_mut_slice(&mut self) -> &mut [Self::State];
}

/// Simple trait for implementations that store states in a Vec of Agent<S>.
/// This assumes that Agent {id, state} is always placed on the id position
/// in vector.
///
/// It automatically provides Population implementations for instances of this
/// trait.
pub trait OwnsContigousAgentSlice {
    type State: State;

    fn as_agent_slice(&self) -> &[Agent<Self::State>];
    fn as_agent_mut_slice(&mut self) -> &mut [Agent<Self::State>];
}
/// A trait for Populations that can grow
pub trait GrowablePopulation: Population {
    /// Spawn an agent in population with the given state and return its handle.
    fn spawn(&mut self, state: Self::State) -> Id;

    /// Spawn many agents in population with the given states and return their handles.
    fn spawns<I: IntoIterator<Item = Self::State>>(&mut self, states: I) -> Vec<Id> {
        let mut ids = Vec::new();
        for state in states {
            ids.push(self.spawn(state));
        }
        return ids;
    }
}

pub trait IterablePopulation<'a>: Population
where
    Self::State: 'a,
{
    type Iter: Iterator<Item = (Id, &'a Self::State)> + 'a;
    type IterMut: Iterator<Item = (Id, &'a mut Self::State)> + 'a;

    /// Iterate over agents
    fn iter_agents(&'a self) -> Self::Iter;

    /// Iterate over agents
    fn iter_agents_mut(&'a mut self) -> Self::IterMut;
}

/////////////////////////////////////////////////////////////////////////////
// Implementations
/////////////////////////////////////////////////////////////////////////////
impl<P> Population for P
where
    P: OwnsStateSlice,
{
    type State = P::State;

    fn from_states<I>(states: I) -> Self
    where
        I: IntoIterator<Item = Self::State>,
    {
        Self::owned_data_from_states(states)
    }

    fn count(&self) -> usize {
        self.as_state_slice().len()
    }

    fn get_agent(&self, id: Id) -> Option<Agent<Self::State>> {
        self.as_state_slice().get(id).map(|ag| Agent {
            id,
            state: ag.clone(),
        })
    }

    fn set_agent(&mut self, id: Id, state: &Self::State) -> &mut Self {
        self.as_state_mut_slice()
            .get_mut(id)
            .map(|st| *st = state.clone());
        return self;
    }

    fn map_agent_mut<B>(&mut self, id: Id, f: impl FnOnce(&mut Self::State) -> B) -> Option<B> {
        self.as_state_mut_slice().get_mut(id).map(f)
    }

    fn map_agent<B>(&self, id: Id, f: impl FnOnce(&Self::State) -> B) -> Option<B> {
        self.as_state_slice().get(id).map(f)
    }

    fn each_agent<F>(&self, f: &mut F)
    where
        F: FnMut(Id, &Self::State),
    {
        for (id, st) in self.as_state_slice().iter().enumerate() {
            f(id, st);
        }
    }

    fn each_agent_mut(&mut self, f: impl FnMut(Id, &mut Self::State)) {
        let mut g = f;
        for (id, st) in self.as_state_mut_slice().iter_mut().enumerate() {
            g(id, st);
        }
    }

    fn random<R: Rng>(&self, rng: &mut R) -> (Id, Self::State) {
        let id = rng.gen_range(0..self.count());
        (id, self.as_state_slice()[id].clone())
    }

    fn randoms_with<T, R, F>(&self, count: usize, rng: &mut R, f: F) -> Vec<(Id, T)>
    where
        R: Rng,
        F: Fn(&Self::State) -> T,
    {
        let mut ids = HashSet::new();
        let mut out = Vec::new();

        while out.len() < count {
            let id = rng.gen_range(0..self.count());
            let state = &self.as_state_slice()[id];
            if !ids.contains(&id) {
                ids.insert(id);
                out.push((id, f(state)));
            }
        }

        return out;
    }
}

impl<S: State> OwnsStateSlice for Vec<S> {
    type State = S;

    fn owned_data_from_states<I>(states: I) -> Self
    where
        I: IntoIterator<Item = Self::State>,
    {
        states.into_iter().collect()
    }

    fn as_state_slice(&self) -> &[S] {
        self.as_slice()
    }

    fn as_state_mut_slice(&mut self) -> &mut [S] {
        self.as_mut_slice()
    }
}

impl<S: State> OwnsContigousAgentSlice for Vec<Agent<S>> {
    type State = S;

    fn as_agent_slice(&self) -> &[Agent<S>] {
        self.as_slice()
    }

    fn as_agent_mut_slice(&mut self) -> &mut [Agent<S>] {
        self.as_mut_slice()
    }
}

// impl<'a, S: State + 'a> IterablePopulation<'a> for Vec<Agent<S>>
// where
//     Self: 'a,
// {
//     // type Iter = Iter<'a, (Id, &'a S)>;
//     // type IterMut = Iter<'a, (Id, &'a mut S)>;

//     fn iter_agents(&'a self) -> Self::Iter {
//         self.iter()
//     }

//     fn iter_agents_mut(&'a mut self) -> Self::IterMut {
//         self.iter_mut()
//     }
// }

impl<'a, S: State> GrowablePopulation for Vec<Agent<S>> {
    fn spawn(&mut self, state: S) -> Id {
        let id = self.len();
        self.push(Agent { id, state });
        return id;
    }
}

impl<S: State> Population for Vec<Agent<S>> {
    type State = S;

    fn from_states<I>(states: I) -> Self
    where
        I: IntoIterator<Item = Self::State>,
    {
        states
            .into_iter()
            .enumerate()
            .map(|(id, state)| Agent { id, state })
            .collect()
    }

    fn count(&self) -> usize {
        self.len()
    }

    fn get_agent(&self, id: Id) -> Option<Agent<Self::State>> {
        self.get(id).map(|ag| ag.clone())
    }

    fn set_agent(&mut self, id: Id, state: &Self::State) -> &mut Self {
        self.get_mut(id).map(|a| a.state = state.clone());
        return self;
    }

    fn map_agent_mut<B>(&mut self, id: Id, f: impl FnOnce(&mut Self::State) -> B) -> Option<B> {
        self.get_mut(id)
            .map(|ag: &mut Agent<Self::State>| f(&mut ag.state))
    }

    fn map_agent<B>(&self, id: Id, f: impl FnOnce(&Self::State) -> B) -> Option<B> {
        self.get(id).map(|ag: &Agent<Self::State>| f(&ag.state))
    }

    fn each_agent<F>(&self, f: &mut F)
    where
        F: FnMut(Id, &Self::State),
    {
        for Agent { id, state } in self.iter() {
            let id = *id;
            f(id, state);
        }
    }

    fn each_agent_mut(&mut self, f: impl FnMut(Id, &mut Self::State)) {
        let mut g = f;
        for Agent { id, state } in self.iter_mut() {
            let id = *id;
            g(id, state);
        }
    }

    fn random<R: Rng>(&self, rng: &mut R) -> (Id, Self::State) {
        let ag = &self[rng.gen_range(0..self.len())];
        (ag.id, ag.state.clone())
    }

    fn randoms_with<T, R, F>(&self, count: usize, rng: &mut R, f: F) -> Vec<(Id, T)>
    where
        R: Rng,
        F: Fn(&Self::State) -> T,
    {
        let mut ids = HashSet::new();
        let mut out = Vec::new();

        while out.len() < count {
            let id = rng.gen_range(0..self.len());
            let state = &self[id].state;
            if !ids.contains(&id) {
                ids.insert(id);
                out.push((id, f(state)));
            }
        }

        return out;
    }
}
