use super::{Population, World};

pub type ReporterList<W, P> = Vec<(usize, Box<dyn Reporter<W, P>>)>;

/// Trait that implements a method that temporarely scans the population and
/// preform some action like collecting statistics, health metrics, emit signals,
/// etc.
///
/// Functions, boxed functions, closures, etc can be executed interpreted as
/// reporters.  
pub trait Reporter<W, P>
where
    W: World,
    P: Population,
{
    /// Register a reporter function to be called every n_steps.
    fn process(&mut self, n: usize, world: &W, population: &P);
}

/// A GrowableReporter can include arbitrary functions that execute during the
/// reporting phase.
pub trait GrowableReporter<W, P>: Reporter<W, P>
where
    W: World,
    P: Population,
{
    /// Register a reporter function to be called every n_steps.
    fn register_reporter(&mut self, n_steps: usize, reporter: Box<dyn Reporter<W, P>>);
}

/////////////////////////////////////////////////////////////////////////////
// Reporter instances
/////////////////////////////////////////////////////////////////////////////

impl<W, P> Reporter<W, P> for fn(&P)
where
    W: World,
    P: Population,
{
    fn process(&mut self, _: usize, _: &W, population: &P) {
        self(population)
    }
}

impl<W, P> Reporter<W, P> for fn(&W, &P)
where
    W: World,
    P: Population,
{
    fn process(&mut self, _: usize, world: &W, population: &P) {
        self(world, population)
    }
}

impl<W, P, F> Reporter<W, P> for F
where
    W: World,
    P: Population,
    F: FnMut(usize, &W, &P),
{
    fn process(&mut self, n: usize, world: &W, population: &P) {
        self(n, world, population)
    }
}

impl<W, P> Reporter<W, P> for ()
where
    W: World,
    P: Population,
{
    fn process(&mut self, _n: usize, _world: &W, _population: &P) {}
}

impl<W, P, R1, R2> Reporter<W, P> for (R1, R2)
where
    W: World,
    P: Population,
    R1: Reporter<W, P>,
    R2: Reporter<W, P>,
{
    fn process(&mut self, n: usize, world: &W, population: &P) {
        self.0.process(n, world, population);
        self.1.process(n, world, population);
    }
}

impl<W, P, R1, R2, R3> Reporter<W, P> for (R1, R2, R3)
where
    W: World,
    P: Population,
    R1: Reporter<W, P>,
    R2: Reporter<W, P>,
    R3: Reporter<W, P>,
{
    fn process(&mut self, n: usize, world: &W, population: &P) {
        self.0.process(n, world, population);
        self.1.process(n, world, population);
        self.2.process(n, world, population);
    }
}

impl<W, P, R1, R2, R3, R4> Reporter<W, P> for (R1, R2, R3, R4)
where
    W: World,
    P: Population,
    R1: Reporter<W, P>,
    R2: Reporter<W, P>,
    R3: Reporter<W, P>,
    R4: Reporter<W, P>,
{
    fn process(&mut self, n: usize, world: &W, population: &P) {
        self.0.process(n, world, population);
        self.1.process(n, world, population);
        self.2.process(n, world, population);
        self.3.process(n, world, population);
    }
}

impl<W, P> Reporter<W, P> for ReporterList<W, P>
where
    W: World,
    P: Population,
{
    fn process(&mut self, n: usize, world: &W, population: &P) {
        for (i, r) in self.iter_mut() {
            if n % (*i) == 0 {
                r.process(n, world, population);
            }
        }
    }
}

/////////////////////////////////////////////////////////////////////////////
// Growable Reporter instances
/////////////////////////////////////////////////////////////////////////////

impl<W, P> GrowableReporter<W, P> for ReporterList<W, P>
where
    W: World,
    P: Population,
{
    fn register_reporter(&mut self, n_steps: usize, reporter: Box<dyn Reporter<W, P>>) {
        self.push((n_steps, reporter));
    }
}

impl<W, P, R1, R2> GrowableReporter<W, P> for (R1, R2)
where
    W: World,
    P: Population,
    R1: Reporter<W, P>,
    R2: GrowableReporter<W, P>,
{
    fn register_reporter(&mut self, n_steps: usize, reporter: Box<dyn Reporter<W, P>>) {
        self.1.register_reporter(n_steps, reporter);
    }
}
