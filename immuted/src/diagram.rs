use const_cstr::*;
use crate::viewmodel::*;
use crate::ui;
use crate::ui::col;
use crate::util::*;
use crate::interlocking::Interlocking;
use crate::dgraph::DGraph;
use crate::canvas::*;
use crate::dispatch::*;
use backend_glfw::imgui::*;
use crate::model::*;
use nalgebra_glm as glm;

pub enum DiagramAction {
    None,
    DraggingCommand(usize)
}

pub struct Diagram { 
    action :DiagramAction,
}


impl Diagram {
    pub fn new() -> Diagram {
        Diagram {
            action: DiagramAction::None,
        }
    }

    fn toolbar(&mut self, doc :&ViewModel, canvas: &mut Canvas) -> Option<()> {
        unsafe {
            let (dispatch_idx,time,play) = canvas.active_dispatch.as_mut()?;
            let view = doc.get_data().dispatch.vecmap_get(*dispatch_idx)?;
            *time = time.max(0.0).min(view.max_t);

            if igButton(if *play { const_cstr!("pause").as_ptr() } else { const_cstr!("play").as_ptr() },
                        ImVec2 { x: 0.0, y: 0.0 }) {
                *play = !*play;
            }

            //let format = const_cstr!("%.3f").as_ptr();
            //igSliderFloat(const_cstr!("Time").as_ptr(), time,
                          //view.time_interval.0, view.time_interval.1, format, 1.0);
            Some(())
        }
    }

    pub fn mouse_pos(&self, doc :&ViewModel, canvas :&Canvas, pos :ImVec2, size :ImVec2) -> Option<ImVec2> {
        unsafe {
            let (dispatch_idx,time,play) = canvas.active_dispatch.as_ref()?;
            let view = doc.get_data().dispatch.vecmap_get(*dispatch_idx)?;
            let io = igGetIO();
            let mousepos = ImVec2 {
                x: glm::lerp_scalar(view.pos_interval.0, view.pos_interval.1, ((*io).MousePos.x - pos.x)/size.x),
                y: glm::lerp_scalar(view.time_interval.0, view.time_interval.1, ((*io).MousePos.y - pos.y)/size.y) };
            Some(mousepos)
        }
    }

    pub fn draw(&mut self, doc :&mut ViewModel, canvas: &mut Canvas) -> Option<()> { unsafe {
        self.toolbar(doc, canvas);
        let mut move_command = None;
        let mut delete_command = None;
        let size = igGetContentRegionAvail_nonUDT2().into();
        ui::canvas(size, const_cstr!("diagramcanvas").as_ptr(), |draw_list, pos| { 

            let mousepos = self.mouse_pos(doc,canvas,pos,size)?;
            let (dispatch_idx,time,play) = canvas.active_dispatch.as_mut()?;

            match self.action {
                DiagramAction::None => {
                    if igIsItemHovered(0) && igIsMouseDown(0) {
                        *time = mousepos.y;
                    }
                },
                DiagramAction::DraggingCommand(cmd_idx) => {
                    if !igIsMouseDragging(0,-1.) { self.action = DiagramAction::None; }
                    let mut new_model = doc.get_undoable().get().clone();
                    if let Some(d) = new_model.dispatches.get_mut(*dispatch_idx) {
                        if let Some((t,_cmd)) = d.0.get_mut(cmd_idx) {
                            if *t != mousepos.y as f64 {
                                *t = mousepos.y as f64;
                                doc.set_model(new_model);
                            }
                        }
                    }
                },
            };


            // Load data for displaying
            let dgraph = doc.get_data().dgraph.as_ref()?;
            let dispatch_spec = doc.get_undoable().get().dispatches.get(*dispatch_idx)?;
            let dispatch = doc.get_data().dispatch.vecmap_get(*dispatch_idx)?;

            Self::time_slider(*time as f64, &dispatch, draw_list, pos, size);

            Self::draw_background(&dispatch, dispatch_spec, draw_list, pos, size);

            let il = doc.get_data().interlocking.as_ref()?;
            Self::command_icons(il, dgraph, &dispatch, dispatch_spec, draw_list, pos, size, 
                                  &mut move_command, &mut delete_command);



            // Things to draw:
            // 1. X front of train (km)
            // 2. back of train (km) (and fill between?)
            // 3. color for identifying trains?
            // 4. color for accel/brake/coast
            // 5. X route activation status?
            // 6. X editable events (train requested, route requested)
            // 7. x detection section blocked
            // 8. scroll/zoom/pan axes
            // 9. signal aspect and sight area
            //
            // Nice tohave:
            // 1. move detector/signals by dragging in diagram (needs reverse-calc km)

            Some(())
        });

        if let Some(cmd_idx) = move_command {
            self.action = DiagramAction::DraggingCommand(cmd_idx);
        }

        if let Some(cmd_idx) = delete_command {
            let mut model = doc.get_undoable().get().clone();
            if let Some((dispatch_idx,time,play)) = canvas.active_dispatch {
                if let Some(d) = model.dispatches.get_mut(dispatch_idx) {
                    d.0.remove(cmd_idx);
                    doc.set_model(model);
                }
            }
        }

        Some(())
    } }

    pub fn time_slider(time :f64, view :&DispatchView, draw_list :*mut ImDrawList, pos: ImVec2, size :ImVec2) {
        unsafe {

            // Draw the line
            ImDrawList_AddLine(draw_list,
                               pos + Self::to_screen(view, &size, time, view.pos_interval.0 as f64),
                               pos + Self::to_screen(view, &size, time, view.pos_interval.1 as f64 ),
                               col::selected(), 2.0);

            let text = format!("t = {:.3}", time);
            ImDrawList_AddText(draw_list, 
                               pos + Self::to_screen(view, &size, time, view.pos_interval.0 as f64),
                               col::unselected(),
                               text.as_ptr() as _ , text.as_ptr().offset(text.len() as isize) as _ );
        }
    }


    pub fn draw_background(view :&DispatchView, dispatch :&Dispatch, draw_list :*mut ImDrawList, pos :ImVec2, size :ImVec2 ) {

        for block in &view.diagram.blocks {
            unsafe {
                // Reserved before
                if block.reserved.0 < block.occupied.0 {
                    ImDrawList_AddRectFilled(draw_list, 
                                             pos + Self::to_screen(view,&size,block.reserved.0, block.pos.0),
                                             pos + Self::to_screen(view,&size,block.occupied.0, block.pos.1),
                                             col::block_a(), 0.0, 0);
                 }

                // Occupied
                ImDrawList_AddRectFilled(draw_list, 
                                         pos + Self::to_screen(view,&size,block.occupied.0, block.pos.0),
                                         pos + Self::to_screen(view,&size,block.occupied.1, block.pos.1),
                                         col::block_b(), 0.0, 0);

                // Reserved after
                if block.reserved.1 > block.occupied.1 {
                    ImDrawList_AddRectFilled(draw_list, 
                                             pos + Self::to_screen(view,&size,block.occupied.1, block.pos.0),
                                             pos + Self::to_screen(view,&size,block.reserved.1, block.pos.1),
                                             col::block_a(), 0.0, 0);

                }

                    ImDrawList_AddRect(draw_list, 
                                             pos + Self::to_screen(view,&size,block.reserved.0, block.pos.0),
                                             pos + Self::to_screen(view,&size,block.reserved.1, block.pos.1),
                                             col::unselected(), 0.0, 0, 1.0);
            }
        }

        for graph in &view.diagram.trains {
            for s in &graph.segments {
                draw_interpolate(draw_list,
                                 pos + Self::to_screen(view, &size, s.start_time, s.kms[0]),
                                 pos + Self::to_screen(view, &size, s.start_time + 1./3.*s.dt , s.kms[1]),
                                 pos + Self::to_screen(view, &size, s.start_time + 2./3.*s.dt , s.kms[2]),
                                 pos + Self::to_screen(view, &size, s.start_time + 3./3.*s.dt , s.kms[3]),
                                 col::unselected()
                                 );
                draw_interpolate(draw_list,
                                 pos + Self::to_screen(view, &size, s.start_time, s.end_kms[0]),
                                 pos + Self::to_screen(view, &size, s.start_time + 1./3.*s.dt , s.end_kms[1]),
                                 pos + Self::to_screen(view, &size, s.start_time + 2./3.*s.dt , s.end_kms[2]),
                                 pos + Self::to_screen(view, &size, s.start_time + 3./3.*s.dt , s.end_kms[3]),
                                 col::unselected_transparent()
                                 );
            }
        }
    }

    pub fn command_icons(il :&Interlocking, dgraph :&DGraph, view :&DispatchView, dispatch :&Dispatch, draw_list :*mut ImDrawList, pos :ImVec2, size :ImVec2, move_command :&mut Option<usize>, delete_command :&mut Option<usize> ) {

        for (idx,(t,cmd)) in dispatch.0.iter().enumerate() {

            let node = match cmd { Command::Route { route, .. } | Command::Train { route, .. } => {
                il.routes.get(*route).and_then(|r| (r.1).first()).map(|(a,b)| a)
            }};
            let km = node.and_then(|n| dgraph.mileage.get(n).cloned()).unwrap_or(0.0);

            unsafe {
                let mouse = (*igGetIO()).MousePos;
                let p = pos + Self::to_screen(view, &size, *t, km);
                let half_icon_size = ImVec2 { x: 4.0, y: 4.0 };
                ImDrawList_AddRectFilled(draw_list, p-half_icon_size, p+half_icon_size, 
                                         ui::col::greenicon(), 0.0, 0);
                if igIsItemHovered(0) && (p-mouse).length_sq() < 5.*5. {
                    igBeginTooltip();
                    ui::show_text(&format!("@{:.3}: {:?}", t, cmd));
                    igEndTooltip();

                    if igIsKeyPressed('D' as _, false ) { 
                        *delete_command = Some(idx);
                    }

                    if igIsMouseDown(0) {
                        *move_command = Some(idx);
                    }
                }

            }
        }
    }

    fn to_screen(dispatch :&DispatchView, size :&ImVec2, t :f64, x :f64) -> ImVec2 {
        ImVec2 { x: size.x*(x as f32 - dispatch.pos_interval.0)
                          /(dispatch.pos_interval.1 - dispatch.pos_interval.0),
                 y: size.y*(t as f32 - dispatch.time_interval.0)
                          /(dispatch.time_interval.1 - dispatch.time_interval.0) }
    }
}

pub fn draw_interpolate(draw_list :*mut ImDrawList, p0 :ImVec2, y1 :ImVec2, y2 :ImVec2, p3 :ImVec2, col:u32) {
    // https://web.archive.org/web/20131225210855/http://people.sc.fsu.edu/~jburkardt/html/bezier_interpolation.html
    let p1 = (-5.0*p0 + 18.0*y1 - 9.0*y2 + 2.0*p3) / 6.0;
    let p2 = (-5.0*p3 + 18.0*y2 - 9.0*y1 + 2.0*p0) / 6.0;
    unsafe {
    ImDrawList_AddBezierCurve(draw_list, p0,p1,p2,p3, col, 2.0, 0);
    }
}

