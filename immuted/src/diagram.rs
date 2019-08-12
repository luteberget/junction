use const_cstr::*;
use crate::viewmodel::*;
use crate::ui;
use crate::util::VecMap;
use crate::canvas::*;
use crate::dispatch::*;
use backend_glfw::imgui::*;

pub struct Diagram { }

impl Diagram {
    pub fn draw(doc :&mut ViewModel, canvas: &mut Canvas) -> Option<()> { unsafe {
        let (dispatch_idx,time) = canvas.active_dispatch.as_mut()?;
        let dgraph = doc.get_data().dgraph.as_ref()?;
        let dispatch = doc.get_data().dispatch.vecmap_get(*dispatch_idx)?;

        let format = const_cstr!("%.3f").as_ptr();
        igSliderFloat(const_cstr!("Time").as_ptr(), time,
                      dispatch.time_interval.0, dispatch.time_interval.1, format, 1.0);

        let size = igGetContentRegionAvail_nonUDT2().into();
        ui::canvas(size, const_cstr!("diagramcanvas").as_ptr(), |draw_list, pos| { 
            Self::draw_background(&dispatch, draw_list, pos, size);

            // Things to draw:
            // 1. X front of train (km)
            // 2. back of train (km) (and fill between?)
            // 3. color for identifying trains?
            // 4. color for accel/brake/coast
            // 5. route activation status
            // 6. editable events (train requested, route requested)
            // 7. detection section blocked
            // 8. scroll/zoom/pan axes
            // 9. signal aspect and sight area
            //
            // Nice tohave:
            // 1. move detector/signals by dragging in diagram (needs reverse-calc km)


        });

        Some(())
    } }


    pub fn draw_background(dispatch :&DispatchView, draw_list :*mut ImDrawList, pos :ImVec2, size :ImVec2) {
        for graph in &dispatch.diagram {
            for s in &graph.segments {
                let p0 = (s.start_time, s.start_pos, s.start_vel);
                let dt = s.dt/3.0;
                let p1 = (p0.0 + dt, p0.1 + p0.2*dt + s.acc*dt*dt*0.5, p0.2 + s.acc*dt);
                let p2 = (p1.0 + dt, p1.1 + p1.2*dt + s.acc*dt*dt*0.5, p1.2 + s.acc*dt);
                let p3 = (p2.0 + dt, p2.1 + p2.2*dt + s.acc*dt*dt*0.5, p2.2 + s.acc*dt);
                draw_interpolate(draw_list,
                                 pos + Self::to_screen(dispatch, &size, p0.0, p0.1),
                                 pos + Self::to_screen(dispatch, &size, p1.0, p1.1),
                                 pos + Self::to_screen(dispatch, &size, p2.0, p2.1),
                                 pos + Self::to_screen(dispatch, &size, p3.0, p3.1));
            }
        }
    }

    fn to_screen(dispatch :&DispatchView, size :&ImVec2, t :f64, x :f64) -> ImVec2 {
        ImVec2 { x: size.x*(t as f32 - dispatch.time_interval.0)
                          /(dispatch.time_interval.1 - dispatch.time_interval.0),
                 y: size.y - size.y*(x as f32 - dispatch.pos_interval.0)
                          /(dispatch.pos_interval.1 - dispatch.pos_interval.0) }
    }
}

pub fn draw_interpolate(draw_list :*mut ImDrawList, p0 :ImVec2, y1 :ImVec2, y2 :ImVec2, p3 :ImVec2) {
    // https://web.archive.org/web/20131225210855/http://people.sc.fsu.edu/~jburkardt/html/bezier_interpolation.html
    let p1 = (-5.0*p0 + 18.0*y1 - 9.0*y2 + 2.0*p3) / 6.0;
    let p2 = (-5.0*p3 + 18.0*y2 - 9.0*y1 + 2.0*p0) / 6.0;
    unsafe {
    ImDrawList_AddBezierCurve(draw_list, p0,p1,p2,p3, ui::col::unselected(), 2.0, 0);
    }
}

