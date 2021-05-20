use covid::{models::*, params::*, prelude::*, sim::*, utils::*};
use rand::{prelude::SmallRng, Rng};
use std::iter;

pub fn main() {
    // type T = models::SEICHAR<()>;
    // type T = models::SEAIR<()>;
    type T = SeirAgent<bool>;
    use simple_logger::SimpleLogger;
    SimpleLogger::new().init().unwrap();

    let mut pop: Vec<T> = new_population(10_000);
    let mut rng_ = default_rng();
    let mut rngb = default_rng();
    let rng = &mut rng_;

    fn score(rng: &mut SmallRng) -> u16 {
        rng.gen()
    }

    pop.distrib_ages(AGE_DISTRIBUTION_BRAZIL, rng)
        // .vaccinate_random(true, 0.9, rng)
        // .distribute_vaccines(2_000, true, move |_| score(&mut rngb))
        // .distribute_vaccines(2_000, true, |ag| ag.age())
        .contaminate_at_random(10, rng)
        .force_infectious(false);

    let params: VaccineDependentSEIR<AgeParam> = Default::default();
    // let mut params = AgeDependentSEIR::<AgeParam>::default();
    let mut sim: Simulation<_, _, _, { T::CARDINALITY }> =
        Simulation::new_simple(params.clone(), pop, 4.5, 0.095);

    sim.seed_from(rng);

    let cases = {
        let mut data = Vec::new();
        data.extend(iter::successors(Some(1.0), |x| Some(1.1 * x)).take(60));
        let last = *data.last().unwrap();
        data.resize_with(90, || last);
        data
    };

    sim.calibrate_sampler_from_cases(&cases);
    sim.run(60);

    // println!("{:?}", cases);
    println!("{}", sim.render_epicurve_csv(T::CSV_HEADER));
    // println!("{:#?}", sim.sample(20));

    // for a in 0..10 {
    //     let obj = sim.get_agent_mut(0).unwrap();
    //     obj.set_age(1 + a * 5);
    //     obj.vaccinate(&false);
    //     println!("{:#?}", obj);
    
    //     let pset = params.clone_to_object(obj);
    //     let p = FullSEIRParams::<Real>::from_universal_params(&pset);
    //     println!("{:#?}", p);
    //     // println!("{:#?}", pset);
    // }
}
