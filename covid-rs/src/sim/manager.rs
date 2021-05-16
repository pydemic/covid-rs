use super::{Population, Reporter, World};

pub trait Manager<W, P, R>
where
    W: World,
    P: Population,
    R: Reporter<W, P>,
{
    /// Compute a final state for the world given an updated population.
    fn update_population(&self, world: &W, population: &mut P);

    /// Compute a final state for the world given an updated population.
    fn update_world(&self, world: &mut W, population: &P);

    /// Start world state and run_reporters the simulation for n_steps.
    fn run_simulation(
        &mut self,
        world: &mut W,
        population: &mut P,
        reporter: &mut R,
        n_steps: usize,
    ) {
        for n in 0..n_steps {
            self.update_population(world, population);
            self.update_world(world, population);
            reporter.process(n, world, population)
        }
    }
}
