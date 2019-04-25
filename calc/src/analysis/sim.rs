use junc_model::scenario::{Dispatch, Command, History};
use junc_model::vehicle::Vehicle;
use rolling::input::staticinfrastructure::{StaticInfrastructure, Routes};

pub fn get_history(vehicles :&[Vehicle], inf :&StaticInfrastructure, 
                   routes :&Routes<usize>, commands :&[(f32, Command)]) -> Result<History, String> {

    // infrastructure and routes are already prepared by the dgraph module
    // we only need to convert commands to the rolling dispatch structs
    // and back from rolling history to glrail history

    use rolling::input::dispatch::DispatchAction;

    let mut dispatch = Vec::new();
    let mut t0 = 0.0;
    let mut train_no = 0;
    for (t,c) in commands {
        if *t > t0 {
            dispatch.push(DispatchAction::Wait(Some((t0-t) as _ )));
            t0 = *t;
        }

        match c {
            Command::Route(r) => dispatch.push(DispatchAction::Route(*r)),
            Command::Train(v,r) => {
                // get train params
                let vehicle = &vehicles[*v];
                let train_params = rolling::railway::dynamics::TrainParams {
                    length: vehicle.length as _,
                    max_acc: vehicle.max_accel as _,
                    max_brk: vehicle.max_brake as _,
                    max_vel: vehicle.max_velocity as _,
                };

                // just make some name for now
                let name = format!("train{}", train_no+1);
                train_no += 1;

                dispatch.push(DispatchAction::Train(name, train_params, *r));
            },
        }
    }

    //println!("Dispatch converted: {:#?}", dispatch);
    //println!(" Running rolling with");
    //println!("infrastructuer : {:?}", inf);
    //println!("routes : {:?}", routes);

    let history = rolling::evaluate_plan(inf, routes, 
             &rolling::input::dispatch::Dispatch { actions: dispatch }, None);

    //println!("History output: {:?}", history);
    // TODO Convert back? Or just keep it like this
    //unimplemented!();

    Ok(history)

}
