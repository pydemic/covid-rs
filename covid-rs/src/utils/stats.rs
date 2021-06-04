use crate::prelude::{Real, INF, NAN};
use getset::*;
use serde::{Deserialize, Serialize};

/// A trait for some data structure that holds statistics about an scalar
/// variable.
///
/// Scalar is easily applicable to a vector of Reals, but also admit other
/// representations that avoid storing the entire dataset in memory if that is
/// not necessary.
pub trait Sampling {
    /// Add a single observation to sample
    fn add(&mut self, x: Real) {
        self.add_many(x, 1);
    }

    /// Add many observations to sample. This can be more efficient than adding
    /// one by one.
    fn add_sequence<I>(&mut self, xs: I) -> &mut Self
    where
        I: Iterator<Item = Real>,
    {
        for x in xs {
            self.add(x);
        }
        return self;
    }

    /// Add many observations to sample from pairs of (data, number of occurrences).
    /// This can be more efficient than adding a single observation
    fn add_sequence_many<I>(&mut self, xs: I) -> &mut Self
    where
        I: Iterator<Item = (Real, usize)>,
    {
        for (x, n) in xs {
            self.add_many(x, n);
        }
        return self;
    }

    /// Add n observations of x
    fn add_many(&mut self, x: Real, n: usize);

    /// Return the sample size
    fn sample_size(&self) -> usize;

    /// Return the sum of all observations
    fn total(&self) -> Real;

    /// Return the minimum value
    fn min(&self) -> Real {
        -INF
    }

    /// Return the maximum value
    fn max(&self) -> Real {
        INF
    }

    /// Return the variance
    fn var(&self) -> Real;

    /// Return the skewness (or normalized third moment)
    fn skew(&self) -> Real;

    /// Return the kurtosis (or normalized fourth moment)
    fn kurt(&self) -> Real;

    /// Return the standard deviation
    fn std(&self) -> Real {
        self.var().sqrt()
    }

    /// Return the mean
    fn mean(&self) -> Real {
        self.total() / self.sample_size() as Real
    }

    /// Return the last sample from distribution.
    fn last_sample(&self) -> Real {
        NAN
    }

    /// Return a simple stats struct holding all statistics as fields.
    fn stats(&self) -> Stats {
        Stats {
            mean: self.mean(),
            std: self.std(),
            skew: self.skew(),
            kurt: self.kurt(),
            min: self.min(),
            max: self.max(),
            size: self.sample_size(),
        }
    }
}

impl Sampling for Vec<Real> {
    fn add_many(&mut self, x: Real, n: usize) {
        for _ in 0..n {
            self.push(x);
        }
    }
    fn total(&self) -> Real {
        self.iter().sum()
    }
    fn var(&self) -> Real {
        StdAcc::from_seq(self.iter().cloned()).var()
    }
    fn skew(&self) -> Real {
        KurtAcc::from_seq(self.iter().cloned()).skew()
    }
    fn kurt(&self) -> Real {
        KurtAcc::from_seq(self.iter().cloned()).kurt()
    }
    fn sample_size(&self) -> usize {
        return self.len();
    }
    fn min(&self) -> Real {
        return self.iter().fold(INF, |acc, x| acc.min(*x));
    }
    fn max(&self) -> Real {
        return self.iter().fold(-INF, |acc, x| acc.max(*x));
    }
    fn stats(&self) -> Stats {
        MinMaxAcc::<KurtAcc>::from_data(self.iter().cloned()).stats()
    }
    fn last_sample(&self) -> Real {
        match self.last() {
            Some(x) => *x,
            _ => NAN,
        }
    }
}

impl Sampling for Vec<(Real, usize)> {
    fn add_many(&mut self, x: Real, n: usize) {
        if let Some(pair) = self.last_mut() {
            if pair.0 == x {
                pair.1 += n;
                return;
            }
        }
        self.push((x, n));
    }
    fn total(&self) -> Real {
        self.iter().map(|(x, n)| x * (*n as Real)).sum()
    }
    fn var(&self) -> Real {
        StdAcc::from_counts(self.iter().cloned()).skew()
    }
    fn skew(&self) -> Real {
        KurtAcc::from_counts(self.iter().cloned()).skew()
    }
    fn kurt(&self) -> Real {
        KurtAcc::from_counts(self.iter().cloned()).kurt()
    }
    fn sample_size(&self) -> usize {
        self.iter().map(|(_, n)| *n).sum()
    }
    fn min(&self) -> Real {
        self.iter().fold(INF, |acc, (x, _)| acc.min(*x))
    }
    fn max(&self) -> Real {
        self.iter().fold(-INF, |acc, (x, _)| acc.max(*x))
    }
    fn stats(&self) -> Stats {
        MinMaxAcc::<KurtAcc>::from_counts(self).stats()
    }
    fn last_sample(&self) -> Real {
        match self.last() {
            Some((x, _)) => *x,
            _ => NAN,
        }
    }
}

/// Accumulate totals and mean. Other values are estimated from a normalized
/// Gaussian distribution
#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct MeanAcc {
    m0: Real,
    m1: Real,
}

impl Sampling for MeanAcc {
    fn add_many(&mut self, x: Real, n: usize) {
        self.m0 += n as Real;
        self.m1 += (n as Real) * x;
    }

    fn sample_size(&self) -> usize {
        self.m0 as usize
    }

    fn total(&self) -> Real {
        self.m1
    }

    fn var(&self) -> Real {
        1.0
    }

    fn skew(&self) -> Real {
        0.0
    }

    fn kurt(&self) -> Real {
        3.0
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct StdAcc {
    m0: Real,
    m1: Real,
    m2: Real,
}

impl Sampling for StdAcc {
    fn add_many(&mut self, x: Real, n: usize) {
        let n = n as Real;
        self.m0 += n;
        self.m1 += x * n;
        self.m2 += x * x * n;
    }

    fn sample_size(&self) -> usize {
        self.m0 as usize
    }

    fn total(&self) -> Real {
        self.m1
    }

    fn var(&self) -> Real {
        let mean = self.m1 / self.m0;
        self.m2 / self.m0 - mean * mean
    }

    fn skew(&self) -> Real {
        0.0
    }

    fn kurt(&self) -> Real {
        todo!()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct KurtAcc {
    m0: Real,
    m1: Real,
    m2: Real,
    m3: Real,
    m4: Real,
}

impl Sampling for KurtAcc {
    fn add_many(&mut self, x: Real, n: usize) {
        let n = n as Real;
        self.m0 += n;
        self.m1 += x * n;
        self.m2 += x * x * n;
        self.m3 += x * x * x * n;
        self.m4 += x * x * x * x * n;
    }
    fn sample_size(&self) -> usize {
        self.m0 as usize
    }
    fn total(&self) -> Real {
        self.m1
    }
    fn var(&self) -> Real {
        let mean = self.m1 / self.m0;
        self.m2 / self.m0 - mean * mean
    }
    fn skew(&self) -> Real {
        let mu = self.mean();
        let std = self.std();
        return (self.m3 / self.m0 - 3.0 * mu * std * std - mu * mu * mu) / (std * std * std);
    }
    fn kurt(&self) -> Real {
        let n = self.m0;
        let (m, b, c, d) = (self.m1 / n, self.m2 / n, self.m3 / n, self.m4 / n);
        let m2 = m * m;
        let m4 = m2 * m2;
        return (d - 4. * m * c + 6. * m2 * b - 3. * m4) / ((b - m2) * (b - m2));
    }
}

/// A simple accumulator of point statistics.
///
/// It stores the latest value computed from number of samples, moments
/// minimum and maximum values and use it to compute descriptive statistics such
/// as the mean, variance, skewness, etc.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct MinMaxAcc<S> {
    acc: S,
    min: Real,
    max: Real,
    last: Real,
}

impl<S: Sampling + Default> MinMaxAcc<S> {
    /// Create new empty Point Stats accumulator
    pub fn new() -> Self {
        Self::default()
    }

    /// Feed iterator into point stats accumulator
    pub fn from_data<'a>(iter: impl Iterator<Item = Real>) -> Self {
        let mut acc = Self::new();
        for x in iter {
            acc.add(x);
        }
        return acc;
    }

    /// Feed iterator of pairs of (x, n_counts) into point stats accumulator
    pub fn from_counts<'a>(iter: impl IntoIterator<Item = &'a (Real, usize)>) -> Self {
        let mut acc = Self::new();
        for (x, n) in iter {
            acc.add_many(*x, *n);
        }
        return acc;
    }

    // /// Merge two accumulators
    // pub fn merge(&self, other: &Self) -> Self {
    //     Accumulator {
    //         n: self.n + other.n,
    //         m1: self.m1 + other.m1,
    //         m2: self.m2 + other.m2,
    //         m3: self.m3 + other.m3,
    //         m4: self.m4 + other.m4,
    //         min: self.min + other.min,
    //         max: self.max + other.max,
    //         last: self.last + other.last,
    //     }
    // }
}

impl<S: Sampling> Sampling for MinMaxAcc<S> {
    fn add_many(&mut self, x: Real, n: usize) {
        self.acc.add_many(x, n);
        self.min = Real::min(x, self.min);
        self.max = Real::max(x, self.max);
        self.last = x;
    }
    fn sample_size(&self) -> usize {
        self.acc.sample_size()
    }
    fn total(&self) -> Real {
        self.acc.total()
    }
    fn min(&self) -> Real {
        self.min
    }
    fn max(&self) -> Real {
        self.max
    }
    fn var(&self) -> Real {
        self.acc.var()
    }
    fn skew(&self) -> Real {
        self.acc.skew()
    }
    fn kurt(&self) -> Real {
        self.acc.kurt()
    }
    fn mean(&self) -> Real {
        self.acc.mean()
    }
    fn last_sample(&self) -> Real {
        self.last
    }
}

impl<S: Default> Default for MinMaxAcc<S> {
    fn default() -> Self {
        MinMaxAcc {
            acc: Default::default(),
            min: INF,
            max: -INF,
            last: NAN,
        }
    }
}

#[inline]
pub fn sqr(x: Real) -> Real {
    x * x
}

/// A simple struct that stores basic values of descriptive statistics about a
/// sample or distribution.
#[derive(Debug, Copy, Clone, PartialEq, Deserialize, Serialize)]
pub struct Stats {
    pub mean: Real,
    pub std: Real,
    pub skew: Real,
    pub kurt: Real,
    pub min: Real,
    pub max: Real,
    pub size: usize,
}

macro_rules! AccImpl {
    ($($ty:ty),*) => {
        $(
            impl $ty {
                pub fn new() -> Self {
                    Self::default()
                }

                pub fn from_seq(seq: impl Iterator<Item = Real>) -> Self {
                    *Self::new().add_sequence(seq)
                }

                pub fn from_counts(seq: impl Iterator<Item = (Real, usize)>) -> Self {
                    *Self::new().add_sequence_many(seq)
                }
            }
        )*
    };
}
AccImpl!(MeanAcc, StdAcc, KurtAcc);
pub type Accumulator = MinMaxAcc<KurtAcc>;

///////////////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn simple_stats() {
        let mut acc = MinMaxAcc::<KurtAcc>::new();
        acc.add(0.);
        acc.add_sequence(vec![1., 2., 3., 4.].iter().cloned());
        let st = acc.stats();
        assert_eq!(st.size, 5);
        assert_approx_eq!(st.mean, 2.0, 0.001);
        assert_approx_eq!(st.std, 1.4142, 0.001);
        assert_approx_eq!(st.skew, 0.0, 0.001);
        assert_approx_eq!(st.kurt, 3.40, 0.001);
    }
}
