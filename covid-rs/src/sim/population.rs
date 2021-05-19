use super::{Agent, DeterministicUpdate, Id, RandomUpdate, World};
use rand::prelude::Rng;
use std::collections::HashSet;

/// The population trait describes a collection of agents.
pub trait Population {
    type State;

    fn from_states<I>(states: I) -> Self
    where
        I: IntoIterator<Item = Self::State>;

    fn to_states(&self) -> Vec<Self::State>
    where
        Self::State: Clone,
    {
        let mut vec = vec![];
        self.each_agent(&mut |_, a: &Self::State| vec.push(a.clone()));
        return vec;
    }

    /// Count the population size.
    fn count(&self) -> usize;

    /// Get an agent by id.
    fn get_agent(&self, id: Id) -> Option<&Self::State>;

    /// Get mutable reference to agent by id.
    fn get_agent_mut(&mut self, id: Id) -> Option<&mut Self::State>;

    /// Get a pair of agents by id.
    fn get_pair(&self, i: Id, j: Id) -> Option<(&Self::State, &Self::State)> {
        match (self.get_agent(i), self.get_agent(j)) {
            (Some(x), Some(y)) => Some((x, y)),
            _ => None,
        }
    }

    /// Get a pair of agents by id.
    fn get_pair_mut(&mut self, i: Id, j: Id) -> Option<(&mut Self::State, &mut Self::State)>;

    /// Get agents by ids. If you need to fetch multiple agents, it can be more
    /// convenient to use this.
    fn get_agents(&self, ids: impl IntoIterator<Item = Id>) -> Vec<(Id, &Self::State)> {
        let mut out = Vec::new();
        for id in ids.into_iter() {
            self.get_agent(id).map(|a| out.push((id, a)));
        }
        return out;
    }

    /// Set an agent state by id.
    fn set_agent(&mut self, id: Id, state: &Self::State) -> &mut Self
    where
        Self::State: Clone,
    {
        self.get_agent_mut(id).map(|st| *st = state.clone());
        return self;
    }

    /// Map function to agent.
    fn map_agent<B>(&self, id: Id, f: impl FnOnce(&Self::State) -> B) -> Option<B> {
        self.get_agent(id).map(f)
    }

    /// Map function to agent, mutating it.
    fn map_agent_mut<B>(&mut self, id: Id, f: impl FnOnce(&mut Self::State) -> B) -> Option<B> {
        self.get_agent_mut(id).map(f)
    }

    /// Set multiple agent states by ids. If you need to update multiple
    /// agents, it can be more convenient to use this.
    fn set_agents(&mut self, updates: &[(Id, &Self::State)]) -> &mut Self
    where
        Self::State: Clone,
    {
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

    /// Select a random id using random number generator.
    fn random_id<R: Rng>(&self, rng: &mut R) -> Id {
        rng.gen_range(0..self.count())
    }

    /// Select a random agent using random number generator.
    fn random<R: Rng>(&self, rng: &mut R) -> (Id, &Self::State)
    where
        Self::State: Clone,
    {
        let i = self.random_id(rng);
        let ag = self.get_agent(i).unwrap();
        return (i, ag);
    }

    /// Select a random agent using random number generator.
    fn random_mut<R: Rng>(&mut self, rng: &mut R) -> (Id, &mut Self::State)
    where
        Self::State: Clone,
    {
        let i = self.random_id(rng);
        let ag = self.get_agent_mut(i).unwrap();
        return (i, ag);
    }

    /// Select many distinct random agents.
    fn randoms<R: Rng>(&self, count: usize, rng: &mut R) -> Vec<(Id, Self::State)>
    where
        Self::State: Clone,
    {
        let mut out = Vec::new();
        self.map_randoms(count, rng, |i, st| out.push((i, st.clone())));
        return out;
    }

    /// Select many distinct random agents.
    fn map_randoms<R, F>(&self, count: usize, rng: &mut R, f: F)
    where
        R: Rng,
        F: FnMut(usize, &Self::State),
        Self::State: Clone,
    {
        let mut ids = HashSet::new();
        let mut missing = count;
        let mut g = f;
        while missing > 0 {
            let (id, state) = self.random(rng);
            if !ids.contains(&id) {
                ids.insert(id);
                g(id, &state);
                missing -= 1;
            }
        }
    }

    /// A mutable version of randoms_with
    fn map_randoms_mut<R, F>(&mut self, count: usize, rng: &mut R, f: F)
    where
        R: Rng,
        F: FnMut(usize, &mut Self::State),
        Self::State: Clone,
    {
        let mut ids = HashSet::new();
        let mut missing = count;
        let mut g = f;
        while missing > 0 {
            let (id, state) = self.random_mut(rng);
            if !ids.contains(&id) {
                ids.insert(id);
                g(id, state);
                missing -= 1;
            }
        }
    }

    /// Apply random_update to each element of population
    fn random_update<W, R>(&mut self, world: &W, rng: &mut R)
    where
        R: Rng,
        W: World,
        <Self as Population>::State: RandomUpdate<W>,
    {
        self.each_agent_mut(&mut |_, st: &mut Self::State| st.random_update(world, rng));
    }

    /// Apply deterministic_update to each element of population
    fn deterministic_update<W>(&mut self, world: &W)
    where
        W: World,
        <Self as Population>::State: DeterministicUpdate<W>,
    {
        self.each_agent_mut(|_, st: &mut Self::State| st.deterministic_update(world));
    }
}

/// Simple trait for implementations that store states in a Vec of states.
///
/// It automatically provides Population implementations for instances of this
/// trait.
pub trait OwnsStateSlice {
    type Elem;

    /// Construct data from iterator.
    fn owned_data_from_states<I>(states: I) -> Self
    where
        I: IntoIterator<Item = Self::Elem>;

    /// Return an immutable slice of agents
    fn as_state_slice(&self) -> &[Self::Elem];

    /// Return a mutable slice of agents
    fn as_state_mut_slice(&mut self) -> &mut [Self::Elem];
}

/// Simple trait for implementations that store states in a Vec of Agent<S>.
/// This assumes that Agent {id, state} is always placed on the id position
/// in vector.
///
/// It automatically provides Population implementations for instances of this
/// trait.
pub trait OwnsContigousAgentSlice {
    type State;

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
    type State = P::Elem;

    fn from_states<I>(states: I) -> Self
    where
        I: IntoIterator<Item = Self::State>,
    {
        Self::owned_data_from_states(states)
    }

    fn count(&self) -> usize {
        self.as_state_slice().len()
    }

    fn get_agent(&self, id: Id) -> Option<&Self::State> {
        self.as_state_slice().get(id)
    }

    fn get_agent_mut(&mut self, id: Id) -> Option<&mut Self::State> {
        self.as_state_mut_slice().get_mut(id)
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

    fn get_pair_mut(&mut self, i: Id, j: Id) -> Option<(&mut Self::State, &mut Self::State)> {
        let slice = self.as_state_mut_slice();
        let n = slice.len();
        if i == j || i >= n || j >= n {
            return None;
        } else {
            // Safety: we can have two mutable borrows to elements of the slice
            // since the previous line guarantees that elements are not the same
            unsafe {
                let a = &mut *(slice.get_unchecked_mut(i) as *mut _);
                let b = &mut *(slice.get_unchecked_mut(j) as *mut _);
                return Some((a, b));
            }
        }
    }
}

impl<S> OwnsStateSlice for Vec<S> {
    type Elem = S;

    fn owned_data_from_states<I>(states: I) -> Self
    where
        I: IntoIterator<Item = Self::Elem>,
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
