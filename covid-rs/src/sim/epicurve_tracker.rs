use super::{Population, Reporter, State, World};
use crate::prelude::Enumerable;
use std::{fmt::Display, ops::Add};

/// A struct with a list of vectors corresponding to the
#[derive(Clone, Debug)]
pub struct EpicurveTracker<T: From<usize>, const N: usize> {
    size: usize,
    data: [Vec<T>; N],
}

impl<T, const N: usize> EpicurveTracker<T, N>
where
    T: From<usize> + Display + Add<Output = T> + Copy + Default,
{
    pub fn new() -> Self {
        unsafe {
            // For now, we use an deprecated method since MaybeUninit is not
            // usable for variable-size arrays yet.
            let mut data: [Vec<T>; N] = std::mem::uninitialized();
            for item in data.iter_mut() {
                std::ptr::write(item, vec![]);
            }
            // unsafe {
            //     let mut data: [MaybeUninit<Vec<usize>>; N] = MaybeUninit::uninit().assume_init();
            //     for ptr in data.iter_mut() {
            //         *ptr = MaybeUninit::new(vec![]);
            //     }
            //     unsafe { std::mem::transmute::<_, [Vec<usize>; N]>(data) }
            // };
            EpicurveTracker {
                size: 0,
                data: data,
            }
        }
    }

    /// Create a new zeroed-counter adding a new empty cell to each column.
    pub fn step(&mut self) {
        for col in &mut self.data {
            col.push(0usize.into());
        }
        self.size += 1;
    }

    /// Increment the counter at i-th column.
    pub fn incr(&mut self, i: usize) {
        if i < N {
            self.data[i][self.size - 1] = self.data[i][self.size - 1] + T::from(1usize);
        }
    }

    /// Update epicurves from population. If step=true, creates a new row and
    /// count elements in each compartment of population. If step=false, it
    /// simply update the current tip.
    pub fn update<P>(&mut self, population: &P, step: bool)
    where
        P: Population,
        P::State: Enumerable,
    {
        if step || self.size == 0 {
            self.step()
        }
        population.each_agent(&mut |_, state: &P::State| self.incr(state.index()));
    }

    /// Return the i-th row.
    pub fn row(&self, i: usize) -> Option<[T; N]> {
        if i >= self.size {
            return None;
        } else {
            let mut out = [T::default(); N];
            for (k, vec) in self.data.iter().enumerate() {
                out[k] = vec[i];
            }
            return Some(out);
        }
    }

    /// Return the i-th column.
    pub fn col(&self, i: usize) -> Option<&[T]> {
        self.data.get(i).map(|ptr| ptr.as_slice())
    }

    /// Return the last row or an array of zeros.
    pub fn tip(&self) -> [T; N]
    where
        T: Default,
    {
        if let Some(Some(data)) = (self.size > 1).then(|| self.row(self.size - 1)) {
            return data;
        }
        return [T::default(); N];
    }

    /// Render epicurves as CSV data
    pub fn render_csv(&self, head: &str, sep: char) -> String {
        let mut data = head.to_string();

        for i in 0..self.size - 1 {
            data.push('\n');
            data.push_str(&format!("{}", self.data[0][i]));
            for j in 1..N {
                data.push(sep);
                data.push_str(&format!("{}", self.data[j][i]));
            }
        }
        return data;
    }
}

impl<T, const N: usize> Default for EpicurveTracker<T, { N }>
where
    T: From<usize> + Display + Add<Output = T> + Copy + Default,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, T, W, P, S, const N: usize> Reporter<W, P> for EpicurveTracker<T, N>
where
    W: World,
    S: Enumerable + State,
    P: Population<State = S>,
    T: From<usize> + Display + Add<Output = T> + Copy + Default,
{
    fn process(&mut self, _n: usize, _world: &W, population: &P) {
        self.update(population, true);
    }
}
