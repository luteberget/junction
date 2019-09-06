type Loc = (f64,f64); // (x, f(x))
#[derive(Debug)]
struct BrentState {
    min :f64,
    max :f64,
    best :Loc,
    second_best :Loc,
    prev_loc :Loc,
    delta :f64,
    prev_delta :f64,
}

fn brent_step(state :&BrentState, rel_move :f64, rel_move2 :f64) -> (f64,f64) {
    let mid = 0.5*(state.min + state.max);
    if state.prev_delta.abs() > rel_move {
        // Try parabolic move
        let r = (state.best.0 - state.second_best.0) * (state.best.1 - state.prev_loc.1);
        let q = (state.best.0 - state.prev_loc.0)    * (state.best.1 - state.second_best.1);
        let p = (state.best.0 - state.prev_loc.0) * q  -  (state.best.0 - state.second_best.0) * r;
        let q = 2.0 * (q - r);
        let p = if q > 0.0 { -p } else { p };
        let q = q.abs();

        if ! (p.abs() >= (q * 0.5*state.prev_delta).abs() 
            || p <= q * (state.min - state.best.0) 
            || p >= q * (state.max - state.best.0)) {

            // parabolic fit ok

            let mut new_delta = p / q;
            let u = state.best.0 + new_delta;
            if u - state.min < rel_move2 || state.max - u < rel_move2 {
                new_delta = if mid - state.best.0 < 0.0 {
                    0.0 - rel_move.abs()
                } else {
                    rel_move.abs()
                };
            }
            //println!("Parabolic");
            return (new_delta, state.delta);
        }
    }

    // Golden section

    let golden = 0.3819660; 
    let new_prev_delta = if state.best.0 >= mid { state.min - state.best.0 } else { state.max - state.best.0 };
    //println!("Golden section");
    (golden*new_prev_delta, new_prev_delta)
}


pub fn brent_minimum(mut f :impl FnMut(f64) -> f64, min :f64, start :f64, max :f64, bits :usize, max_iter :Option<usize>) -> (f64,f64) {
    println!("Brent minimum {} {} {} {:?}", min, max, bits, max_iter);
    // ported from boost 1.63 boost/math/tools/minima.hpp
    let tolerance = (1.0 - bits as f64).exp2(); // 2^-(1-bits)
    
    let mut state = {
        println!("calling1");
        let fstart = f(start);
        println!("calling1 ok");
        BrentState {
            min: min,
            max: max,
            best: (start, fstart),
            second_best: (start, fstart),
            prev_loc: (start, fstart),
            delta: 0.0,
            prev_delta: 0.0,
        }
    };

    let mut iters = max_iter.unwrap_or(1_000_000_000);

    loop {
        //println!("Iteration {:?}", state);
        let mid = 0.5*(state.min + state.max); // midpoint of current interval

        // check if we are done
        let rel_move  = tolerance*( 0.25 + state.best.0.abs() );
        let rel_move2 = 2.0 * rel_move;
        //println!("Checking termination {} <= {}", (state.best.0 - mid).abs(), (rel_move2 - 0.5*(state.max - state.min)));
        if (state.best.0 - mid).abs() <= (rel_move2 - 0.5*(state.max - state.min)) {
            break;
        }

        // choose between parabolic and golden section
        let deltas = brent_step(&state, rel_move, rel_move2);
        state.delta = deltas.0;
        state.prev_delta = deltas.1;

        // update position
        let next_param = if state.delta.abs() >= rel_move {
            state.best.0 + state.delta
        } else {
            if state.delta > 0.0 {
                state.best.0 + rel_move.abs()
            } else {
                state.best.0 - rel_move.abs()
            }
        };

        println!("calling2");
        let next_loc = (next_param, f(next_param));
        println!("calling2 ok");

        if next_loc.1 <= state.best.1 {
            // best point seen so far
            if next_loc.0 >= state.best.0  {
                state.min = state.best.0;
            } else {
                state.max = state.best.0;
            }

            state.prev_loc = state.second_best;
            state.second_best = state.best;
            state.best = next_loc;
        } else {
            // The point was worse, but it must be better than one of the end points.
            if next_loc.0 < state.best.0 {
                state.min = next_loc.0;
            } else {
                state.max = next_loc.0;
            }

            if next_loc.1 <= state.second_best.1 || state.second_best.0 == state.best.0 {
                // Second best rel_move2
                state.prev_loc = state.second_best;
                state.second_best = next_loc;
            } else if next_loc.1 <= state.prev_loc.1 
                || state.prev_loc.0 == state.best.0 
                || state.prev_loc.0 == state.second_best.0 {
                    // Third best
                    state.prev_loc = next_loc;
                }
        }

        iters -= 1;
        if iters == 0 { break; }
    }

    state.best
}



#[cfg(test)]
mod tests {
    #[test]
    fn test1() {

        // example from  https://www.boost.org/doc/libs/1_63_0/libs/math/doc/html/math_toolkit/roots/brent_minima.html
        //
        //

        use crate::*;
        let f  = |x| (x + 3.0)*(x - 1.0)*(x - 1.0); // (x+3)(x-1)^2
        let r = brent_minimum(f, 0.0, 0.5, 4.0 / 3.0, 50, None);
        println!("Minimum of (x+3)(x-1)^2 in interval (-4, 4/3) = {:?}", r);

    }
}
