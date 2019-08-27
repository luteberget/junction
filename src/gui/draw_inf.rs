use crate::app::*;
use crate::gui::widgets::Draw;
use crate::util;
use crate::document::model::*;
use crate::document::objects::*;
use crate::document::infview::*;
use crate::config::*;

use backend_glfw::imgui::*;
use nalgebra_glm as glm;
use matches::*;
use std::collections::HashMap;

pub fn base(app :&mut App, draw :&Draw) { 
    let empty_state = HashMap::new();
    let mut object_states :&HashMap<PtA,Vec<ObjectState>> = &empty_state;

    // TODO get instant
    //if let Some((idx,time,_play)) = self.active_dispatch.as_ref() {
    //    if let Some(instant) = self.instant_cache.get_cached_instant(vm, *idx, *time) {
    //        object_states = &instant.infrastructure.object_state;
    //    }
    //}

    let doc = &app.document;
    let m = doc.model();
    let d = doc.data();
    let inf_view = &doc.inf_view;
    let config = &app.config;

    unsafe {

        let sel_window = if let Action::Normal(NormalState::SelectWindow(a)) = &inf_view.action {
            Some((*a, *a + igGetMouseDragDelta_nonUDT2(0,-1.0).into()))
        } else { None };

        let (lo,hi) = inf_view.view.points_in_view(draw.size);
        let color_grid = config.color_u32(RailUIColorName::CanvasGridPoint);
        for x in lo.x..=hi.x {
            for y in lo.y..=hi.y {
                let pt = inf_view.view.world_pt_to_screen(glm::vec2(x,y));
                ImDrawList_AddCircleFilled(draw.draw_list, draw.pos+pt, 3.0, color_grid, 4);
            }
        }

        let color_line = config.color_u32(RailUIColorName::CanvasTrack);
        let color_line_selected = config.color_u32(RailUIColorName::CanvasTrackSelected);
        for l in &m.linesegs {
            let p1 = inf_view.view.world_pt_to_screen(l.0);
            let p2 = inf_view.view.world_pt_to_screen(l.1);
            let selected = inf_view.selection.contains(&Ref::LineSeg(l.0,l.1));
            let preview = sel_window
                .map(|(a,b)| util::point_in_rect(p1,a,b) || util::point_in_rect(p2,a,b))
                .unwrap_or(false) ;
            let col = if selected || preview { color_line_selected } else { color_line };
            ImDrawList_AddLine(draw.draw_list, draw.pos + p1, draw.pos + p2, col, 2.0);
        }

        let color_node = config.color_u32(RailUIColorName::CanvasNode);
        let color_node_selected = config.color_u32(RailUIColorName::CanvasNodeSelected);
        if let Some(topo) = d.topology.as_ref() {
            use nalgebra_glm::{vec2, rotate_vec2, radians, vec1, normalize};
            for (pt0,(t,vc)) in &topo.locations {
                let selected = inf_view.selection.contains(&Ref::Node(*pt0));
                let preview = sel_window.map(|(a,b)| 
                         util::point_in_rect(inf_view.view.world_pt_to_screen(*pt0),a,b)).unwrap_or(false);
                let col = if selected || preview { color_node_selected } 
                            else { color_node };

                let pt :PtC = vec2(pt0.x as _ ,pt0.y as _ );
                let tangent :PtC = vec2(vc.x as _ ,vc.y as _ );
                match t {
                    NDType::OpenEnd => {
                        for angle in &[-45.0,45.0] {
                            ImDrawList_AddLine(draw.draw_list,
                               draw.pos + inf_view.view.world_ptc_to_screen(pt),
                               draw.pos + inf_view.view.world_ptc_to_screen(pt) 
                                + util::to_imvec(8.0*rotate_vec2(&normalize(&tangent),radians(&vec1(*angle)).x)), col, 2.0);
                        }
                    },
                    NDType::Cont => {
                        ImDrawList_AddCircleFilled(draw.draw_list, 
                            draw.pos + inf_view.view.world_ptc_to_screen(pt), 4.0, col, 8);
                    },
                    NDType::Sw(side) => {
                        let angle = if matches!(side, Side::Left) { 45.0 } else { -45.0 };
                        let p1 = draw.pos + inf_view.view.world_ptc_to_screen(pt);
                        let p2 = p1 + util::to_imvec(15.0*normalize(&tangent));
                        let p3 = p1 + util::to_imvec(15.0*rotate_vec2(&(1.41*normalize(&tangent)), radians(&vec1(angle)).x));
                        ImDrawList_AddTriangleFilled(draw.draw_list, p1,p2,p3, col);
                    },
                    NDType::Err =>{
                        let p = draw.pos + inf_view.view.world_ptc_to_screen(pt);
                        let window = ImVec2 { x: 4.0, y: 4.0 };
                        ImDrawList_AddRect(draw.draw_list, p - window, p + window,
                                           config.color_u32(RailUIColorName::CanvasNodeError),
                                           0.0,0,4.0);
                    },
                    NDType::BufferStop => {
                        let tangent = util::to_imvec(normalize(&tangent));
                        let normal = ImVec2 { x: -tangent.y, y: tangent.x };

                        let node = draw.pos + inf_view.view.world_ptc_to_screen(pt);
                        let pline :&[ImVec2] = &[node + 8.0*normal + 4.0 * tangent,
                                              node + 8.0*normal,
                                              node - 8.0*normal,
                                              node - 8.0*normal + 4.0 * tangent];

                        ImDrawList_AddPolyline(draw.draw_list,pline.as_ptr(), pline.len() as i32, col, false, 2.0);

                    },
                    NDType::Crossing(type_) => {
                        let left_conn  = matches!(type_, CrossingType::DoubleSlip | CrossingType::SingleSlip(Side::Left));
                        let right_conn = matches!(type_, CrossingType::DoubleSlip | CrossingType::SingleSlip(Side::Right));

                        let tangenti = util::to_imvec(normalize(&tangent));
                        let normal = ImVec2 { x: tangenti.y, y: tangenti.x };

                        if right_conn {
                            let base = draw.pos + inf_view.view.world_ptc_to_screen(pt) - 4.0*normal - 2.0f32.sqrt()*2.0*tangenti;
                            let pline :&[ImVec2] = &[base - 8.0*tangenti,
                                                     base,
                                                     base + 8.0*util::to_imvec(rotate_vec2(&tangent, radians(&vec1(45.0)).x))];
                            ImDrawList_AddPolyline(draw.draw_list,pline.as_ptr(), pline.len() as i32, col, false, 2.0);
                        }

                        if left_conn {
                            let base = draw.pos + inf_view.view.world_ptc_to_screen(pt) + 4.0*normal + 2.0f32.sqrt()*2.0*tangenti;
                            let pline :&[ImVec2] = &[base + 8.0*tangenti,
                                                     base,
                                                     base - 8.0*util::to_imvec(rotate_vec2(&tangent, radians(&vec1(45.0)).x))];
                            ImDrawList_AddPolyline(draw.draw_list,pline.as_ptr(), pline.len() as i32, col, false, 2.0);
                        }

                        if left_conn || right_conn {
                            let p = draw.pos + inf_view.view.world_ptc_to_screen(pt);
                            let pa = util::to_imvec(15.0*normalize(&tangent));
                            let pb = util::to_imvec(15.0*rotate_vec2(&normalize(&tangent), radians(&vec1(45.0)).x));
                            ImDrawList_AddTriangleFilled(draw.draw_list,p,p+pa,p+pb,col);
                            ImDrawList_AddTriangleFilled(draw.draw_list,p,p-pa,p-pb,col);
                        } else {
                            ImDrawList_AddCircleFilled(draw.draw_list, draw.pos + inf_view.view.world_ptc_to_screen(pt), 4.0, col, 8);
                        }
                    },
                }
            }
        }


        let color_obj = config.color_u32(RailUIColorName::CanvasSymbol);
        let color_obj_selected = config.color_u32(RailUIColorName::CanvasSymbolSelected);

        for (pta,obj) in &m.objects {
            let selected = inf_view.selection.contains(&Ref::Object(*pta));
            let preview = sel_window.map(|(a,b)| 
                     util::point_in_rect(inf_view.view.
                             world_ptc_to_screen(unround_coord(*pta)),a,b)).unwrap_or(false);
            let col = if selected || preview { color_obj_selected } else { color_obj };
            let empty = vec![];
            let state = object_states.get(pta).unwrap_or(&empty);
            obj.draw(draw.pos, &inf_view.view, draw.draw_list, col, state, config);
        }
    }
}

pub fn route(app :&mut App, draw :&Draw, r :usize) { 
}

pub fn trains(app :&mut App, draw :&Draw) { 
}

pub fn state(app :&mut App, draw :&Draw) { 
}

