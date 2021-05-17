use covid::{epidemic::*, models, prelude::*, sim::*};
use rand::{
    prelude::{SeedableRng, SmallRng},
    Rng,
};
use std::cell::RefCell;

fn juxt<'a, A: 'a, B: 'a>(
    f1: Box<dyn FnMut(A, B) + 'a>,
    f2: Box<dyn FnMut(A, B) + 'a>,
) -> Box<dyn FnMut(A, B) + 'a>
where
    A: Copy,
    B: Copy,
{
    let mut g1 = f1;
    let mut g2 = f2;

    Box::new(move |w, p| {
        g1(w, p);
        g2(w, p);
    }) as Box<dyn FnMut(A, B)>
}

pub struct Simulation<W, P, S, const N: usize> {
    population: P,
    world: W,
    sampler: S,
    reporter: EpicurveReporter<W, P, { N }>,
    world_update: Box<dyn FnMut(&mut W, &P)>,
    population_update: Box<dyn FnMut(&W, &mut P)>,
    rng: RefCell<SmallRng>,
}

impl<W, P, S, const N: usize> Simulation<W, P, S, { N }>
where
    W: World,
    P: Population,
    S: Sampler<P>,
    P::State: StochasticUpdate<W> + SIRLike,
{
    pub fn new(world: W, population: P, sampler: S) -> Self {
        let reporter = EpicurveReporter::new(&population);
        Simulation {
            population,
            world,
            sampler,
            reporter: reporter,
            world_update: Box::new(|_: &mut W, _: &P| {}),
            population_update: Box::new(|_: &W, _: &mut P| {}),
            rng: RefCell::new(SmallRng::from_entropy()),
        }
    }

    pub fn run(&mut self, n_steps: usize) {
        let rng = &mut *self.rng.borrow_mut();
        let population_update = &mut self.population_update;
        let world_update = &mut self.world_update;
        let sampler = &self.sampler;

        for n in 0..n_steps {
            // Default updates
            self.population.update_random(&self.world, rng);
            let cases = sampler.update_epimodel_population(&mut self.population, rng);

            // Arbitrary updates
            population_update(&self.world, &mut self.population);
            world_update(&mut self.world, &self.population);
            self.reporter.process(n, &self.world, &self.population)
        }
    }

    pub fn render_epicurve_csv(&self, head: &str) -> String {
        self.reporter.render_epicurve_csv(head)
    }
}

impl<W, P, const N: usize> Simulation<W, P, SimpleSampler, { N }>
where
    W: World,
    P: Population,
    P::State: StochasticUpdate<W> + SIRLike,
{
    pub fn new_simple(world: W, population: P, n_contacts: Real, prob_infection: Real) -> Self {
        let sampler = SimpleSampler::new(n_contacts, prob_infection);
        return Self::new(world, population, sampler);
    }
}

pub fn main() {
    type T = models::SEICHAR<()>;
    // type T = models::SEAIR<()>;
    use simple_logger::SimpleLogger;
    SimpleLogger::new().init().unwrap();

    let mut pop: Vec<Agent<T>> = new_population(10000);
    let mut params = Params::default();

    // Infect elements
    pop.set_agents(&[
        (0, &T::new_infectious()),
        (1, &T::new_infectious()),
        (2, &T::new_infectious()),
        (3, &T::new_infectious()),
        (4, &T::new_infectious()),
        (5, &T::new_infectious()),
        (6, &T::new_infectious()),
        (7, &T::new_infectious()),
        (8, &T::new_infectious()),
        (9, &T::new_infectious()),
    ]);

    let mut sim: Simulation<_, _, _, { T::CARDINALITY }> =
        Simulation::new_simple(params, pop, 4.5, 0.095);
    sim.run(180);
    // println!("{:#?}", pop);
    // println!("{:#?}", params);
    println!("{}", sim.render_epicurve_csv("S,E,A,I,R,D"));

    use std::mem::size_of;
    println!("SIR_t: {}", size_of::<models::SEIR<()>>());
    println!("usize: {}", size_of::<usize>());
    println!("bool: {}", size_of::<bool>());
}
