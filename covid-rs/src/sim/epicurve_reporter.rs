use super::{EpicurveTracker, GrowableReporter, Population, Reporter, ReporterList, World};
use crate::prelude::{Enumerable, SIR};
use std::fmt::Debug;

/// Epicurve reporter that can be extended with an arbitrary list of FnMut()
/// reporters.
pub struct EpicurveReporter<W, P, const N: usize> {
    n_iter: usize,
    epicurves: EpicurveTracker<usize, { N }>,
    reporters: ReporterList<W, P>,
}

impl<W, P, const N: usize> EpicurveReporter<W, P, { N }> {
    pub fn new(population: &P) -> Self
    where
        P: Population,
        P::State: SIR,
    {
        let mut new = EpicurveReporter {
            n_iter: 0,
            reporters: vec![],
            epicurves: Default::default(),
        };
        new.epicurves.update(population, true);
        return new;
    }

    /// Return a CSV string with the content of the Epicurves.
    pub fn render_epicurve_csv(&self, head: &str) -> String {
        self.epicurves.render_csv(head, ',')
    }

    /// Return an arrray with the last row of the Epicurve.
    pub fn epicurve_tip(&self) -> [usize; N] {
        self.epicurves.tip()
    }
}

impl<W, P, const N: usize> Debug for EpicurveReporter<W, P, { N }> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EpiReporter")
            .field("epicurves", &self.epicurves)
            .finish()
    }
}

impl<W, P, const N: usize> Reporter<W, P> for EpicurveReporter<W, P, { N }>
where
    W: World,
    P: Population,
    P::State: Enumerable,
{
    fn process(&mut self, n: usize, world: &W, population: &P) {
        self.epicurves.process(n, world, population);
        self.reporters.process(n, world, population);
        self.n_iter += 1;
    }
}

impl<W, P, const N: usize> GrowableReporter<W, P> for EpicurveReporter<W, P, { N }>
where
    W: World,
    P: Population,
    P::State: Enumerable,
{
    fn register_reporter(&mut self, n_steps: usize, reporter: Box<dyn Reporter<W, P>>) {
        self.reporters.register_reporter(n_steps, reporter)
    }
}
