use super::{Population, Reporter};
use crate::prelude::EpiModel;
use std::{fmt::Display, ops::Add};

/// A struct with a list of vectors corresponding to the
#[derive(Clone, Debug)]
pub struct EpicurveTracker<T: From<usize>, const N: usize> {
    size: usize,
    data: [Vec<T>; N],
}

impl<T, const N: usize> EpicurveTracker<T, { N }>
where
    T: From<usize> + Display + Add<Output = T> + Copy + Default,
{
    pub fn new() -> Self {
        // Cannot use this code because of dependently sized N
        // EpicurveTracker {
        //     size: 0,
        //     data: unsafe {
        //         let mut data: [MaybeUninit<Vec<T>>; N] = MaybeUninit::uninit().assume_init();
        //         for ptr in data.iter_mut() {
        //             *ptr = MaybeUninit::new(vec![]);
        //         }
        //         std::mem::transmute::<_, [Vec<T>; N]>(data)
        //     },
        // }

        // SAFETY
        // We need to create an array of empty vectors fo arbitrary size. Usually
        // this would be done as [Vec::default(); N], but Rust does not support
        // this with N >= 32, or if the size of N comes from a compilation constant
        // that is yet unknown.
        unsafe {
            // For now, we use an deprecated method since MaybeUninit is not
            // usable for variable-size arrays yet.
            let mut data: [Vec<T>; N] = std::mem::uninitialized();
            for item in data.iter_mut() {
                std::ptr::write(item, vec![]);
            }

            EpicurveTracker {
                size: 0,
                data: data,
            }
        }
    }

    /// Merge two epicurve trackers
    pub fn merge<const M: usize>(
        &self,
        other: &EpicurveTracker<T, M>,
    ) -> Option<EpicurveTracker<T, { N + M }>> {
        if self.size != other.size {
            return None;
        }
        let mut out = EpicurveTracker::new();
        out.size = self.size;
        for j in 0..N {
            out.data[j] = self.data[j].clone();
        }
        for j in 0..M {
            out.data[j] = other.data[j].clone();
        }
        return Some(out);
    }

    /// Add new column to tracker
    pub fn with_column(&self, data: &Vec<T>) -> Option<EpicurveTracker<T, { N + 1 }>> {
        self.merge(&EpicurveTracker::<T, 1>::from(data))
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
        P::State: EpiModel,
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
    S: EpiModel,
    P: Population<State = S>,
    T: From<usize> + Display + Add<Output = T> + Copy + Default,
{
    fn process(&mut self, _n: usize, _world: &W, population: &P) {
        self.update(population, true);
    }
}

impl<T> From<Vec<T>> for EpicurveTracker<T, 1>
where
    T: From<usize>,
{
    fn from(data: Vec<T>) -> Self {
        EpicurveTracker {
            size: data.len(),
            data: [data],
        }
    }
}

impl<T> From<&Vec<T>> for EpicurveTracker<T, 1>
where
    T: From<usize> + Clone,
{
    fn from(data: &Vec<T>) -> Self {
        EpicurveTracker {
            size: data.len(),
            data: [data.clone()],
        }
    }
}

