use imgui_sys_bindgen::sys::*;
use imgui_sys_bindgen::json::*;
use imgui_sys_bindgen::text::*;
use crate::app::*;
use crate::model::*;
use crate::scenario::*;
use crate::infrastructure::*;
use crate::selection::*;
use crate::dgraph::*;
use crate::command_builder::*;
use crate::view::*;
use crate::graph::*;
use std::ptr;
use std::ffi::CString;
use const_cstr::const_cstr;

use imgui_sys_bindgen::sys::ImVec2;
pub fn world2screen(topleft: ImVec2, bottomright: ImVec2, center :(f64,f64), zoom: f64, pt :(f32,f32)) -> ImVec2 {
    let scale = if bottomright.x - topleft.x < bottomright.y - topleft.y {
        (bottomright.x-topleft.x) as f64 / zoom
    } else {
        (bottomright.y-topleft.y) as f64 / zoom
    };
    let x = 0.5*(topleft.x + bottomright.x) as f64 + scale*(pt.0 as f64  - center.0);
    let y = 0.5*(topleft.y + bottomright.y) as f64 + scale*(-(pt.1 as f64 -  center.1));
    ImVec2 {x: x as _ , y: y as _ }
}

pub fn screen2world(topleft: ImVec2, bottomright: ImVec2, center: (f64, f64), zoom: f64, pt:ImVec2) -> (f32,f32) {
    let scale = if bottomright.x - topleft.x < bottomright.y - topleft.y {
        (bottomright.x-topleft.x) as f64 / zoom
    } else {
        (bottomright.y-topleft.y) as f64 / zoom
    };
    // mousex = 0.5 tlx + 0.5 brx + scale*ptx - scale*cx
    // ptx = 1/scale*(mousex - 0.5tlx - 0.5brx + scale*cx)
    let x = 1.0/scale*(pt.x as f64 - (0.5*(topleft.x + bottomright.x)) as f64) + center.0;
    let y = 1.0/scale*(pt.y as f64 - (0.5*(topleft.y + bottomright.y)) as f64) + center.1;
    (x as _,(-y) as _ )
}

pub fn screen2worldlength(topleft: ImVec2, bottomright: ImVec2, zoom: f64, d :f32) -> f32 {
    let scale = if bottomright.x - topleft.x < bottomright.y - topleft.y {
        (bottomright.x-topleft.x) as f64 / zoom
    } else {
        (bottomright.y-topleft.y) as f64 / zoom
    };

    ((d as f64)/scale) as f32
}

pub fn  line_closest_pt(a :&ImVec2, b :&ImVec2, p :&ImVec2) -> ImVec2 {
    let ap = ImVec2{ x: p.x - a.x, y:  p.y - a.y};
    let ab_dir = ImVec2 { x: b.x - a.x, y: b.y - a.y };
    let dot = ap.x * ab_dir.x + ap.y * ab_dir.y;
    if dot < 0.0 { return *a; }
    let ab_len_sqr = ab_dir.x * ab_dir.x + ab_dir.y * ab_dir.y;
    if dot > ab_len_sqr { return *b; }
    let ac = ImVec2{ x: ab_dir.x * dot / ab_len_sqr, y: ab_dir.y * dot / ab_len_sqr } ;
    ImVec2 { x : a.x + ac.x, y: a.y + ac.y }
}

pub fn dist2(a :&ImVec2, b :&ImVec2) -> f32 { 
    (a.x - b.x)*(a.x - b.x) + (a.y - b.y)*(a.y - b.y)
}


pub fn canvas(mainmain_size: ImVec2, app :&mut App) -> bool {
    let v2_0 = ImVec2 { x: 0.0, y: 0.0 };

        let canvas_bg = 60 + (60<<8) + (60<<16) + (255<<24);
    let line_col  = 208 + (208<<8) + (175<<16) + (255<<24);
    let tvd_col  = 175 + (255<<8) + (175<<16) + (255<<24);
    let selected_col  = 175 + (175<<8) + (255<<16) + (255<<24);
    let line_hover_col  = 255 + (50<<8) + (50<<16) + (255<<24);
    // TODO make some colors config struct

    let occupied_col  = 255 + (65<<8) + (55<<16) + (255<<24);
    let reserved_col  = 55 + (255<<8) + (55<<16) + (255<<24);
    let overlap_col  = 209 + (208<<8) + (22<<16) + (255<<24);

    unsafe {


                      // TODO move this out of main loop
                let caret_right = const_cstr!("\u{f0da}");
                let caret_left = const_cstr!("\u{f0d9}");
                let (caret_left_halfsize,caret_right_halfsize) = unsafe {
                    let mut l = igCalcTextSize(caret_left.as_ptr(), ptr::null(), false, -1.0);
                    let mut r = igCalcTextSize(caret_right.as_ptr(), ptr::null(), false, -1.0);
                    l.x *= 0.5; l.y *= 0.5; r.x *= 0.5; r.y *= 0.5;
                    (l,r)
                };



    let io = igGetIO();
    let mouse_pos = (*io).MousePos;

  igBeginChild(const_cstr!("Canvas").as_ptr(), mainmain_size, false, 0);
  let capture_canvas_key = igIsWindowFocused(0);

  let draw_list = igGetWindowDrawList();
  //igText(const_cstr!("Here is the canvas:").as_ptr());

  match &app.model.schematic {
      Derive::Wait => {
          igText(const_cstr!("Solving...").as_ptr());
      },
      Derive::Err(ref e) => {
          let s = CString::new(format!("Error: {}", e)).unwrap();
          igText(s.as_ptr());
      },
      Derive::Ok(ref s) => {
          let mut hovered_item = None;
          let canvas_pos = igGetCursorScreenPos();
          let mut canvas_size = igGetContentRegionAvail();
          let canvas_lower = ImVec2 { x: canvas_pos.x + canvas_size.x,
                                      y: canvas_pos.y + canvas_size.y };
          if canvas_size.x < 10.0 { canvas_size.x = 10.0 }

          if canvas_size.y < 10.0 { canvas_size.y = 10.0 }
          ImDrawList_AddRectFilled(draw_list, canvas_pos,
                                   ImVec2 { x: canvas_pos.x + canvas_size.x,
                                            y: canvas_pos.y + canvas_size.y, },
                                            canvas_bg,
                                    0.0, 0);
          let clicked = igInvisibleButton(const_cstr!("canvasbtn").as_ptr(), canvas_size);
          let right_clicked = igIsItemHovered(0) && igIsMouseClicked(1,false);
          let canvas_hovered = igIsItemHovered(0);

          let (center,zoom) = app.model.view.viewport;

          if igIsItemActive() && igIsMouseDragging(0,-1.0) {
              (app.model.view.viewport.0).0 -= screen2worldlength(canvas_pos, canvas_lower, zoom, (*io).MouseDelta.x) as f64;
              (app.model.view.viewport.0).1 += screen2worldlength(canvas_pos, canvas_lower, zoom, (*io).MouseDelta.y) as f64;
          }

          if igIsItemHovered(0) {
              let wheel = (*io).MouseWheel;
              //println!("{}", wheel);
              let wheel2 = 1.0-0.2*(*io).MouseWheel;
              //println!("{}", wheel2);
              (app.model.view.viewport.1) *= wheel2 as f64;
          }
          

          // Iterate the schematic 


          ImDrawList_PushClipRect(draw_list, canvas_pos, canvas_lower, true);

          let mut lowest = std::f32::INFINITY;

          for (k,v) in &s.lines {
              //println!("{:?}, {:?}", k,v);
              let mut hovered = false;
              let selected = if let Selection::Entity(EntityId::Track(id)) = &app.model.view.selection { id == k } else { false };
              for i in 0..(v.len()-1) {
                  let p1 = world2screen(canvas_pos, canvas_lower, center, zoom, v[i]);
                  let p2 = world2screen(canvas_pos, canvas_lower, center, zoom, v[i+1]);
                  let hovered = dist2(&mouse_pos, &line_closest_pt(&p1, &p2, &mouse_pos)) < 100.0;
                  if hovered {
                      hovered_item = Some(EntityId::Track(*k));
                  }
                  ImDrawList_AddLine(draw_list, p1, p2, 
                                     if selected { selected_col }
                                     else if canvas_hovered && hovered { line_hover_col } else { line_col }, 2.0);
                  lowest = lowest.min(v[i].1);
                  lowest = lowest.min(v[i+1].1);
              }
          }

          // Example plot of a detection section 
          // TODO trigger by selecting/hovering routes in the menu
          if let Derive::Ok(DGraph { tvd_sections, edge_intervals, .. }) = &app.model.dgraph {
              if let Some((sec_id, edges)) = tvd_sections.iter().nth(0) {
                  for e in edges.iter() {
                      if let Some(Interval { track, p1, p2 }) = edge_intervals.get(e) {

                          if let Some((loc1,_)) = s.track_line_at(track, *p1) {
                          if let Some((loc2,_)) = s.track_line_at(track, *p2) {

                              let ps1 = world2screen(canvas_pos, canvas_lower, center, zoom, loc1);
                              let ps2 = world2screen(canvas_pos, canvas_lower, center, zoom, loc2);
                              ImDrawList_AddLine(draw_list, ps1,ps2, tvd_col, 5.0);

                          }
                          }
                      }
                  }
              }
          }


          for (k,v) in &s.points {
              let mut p = world2screen(canvas_pos, canvas_lower, center, zoom, *v);
              let tl = ImVec2 { x: p.x - caret_right_halfsize.x, 
                                 y: p.y - caret_right_halfsize.y };
              let br = ImVec2 { x: p.x + caret_right_halfsize.x, 
                                 y: p.y + caret_right_halfsize.y };
              let symbol = match app.model.inf.get_node(k) {
                  Some(Node(_,NodeType::BufferStop)) => caret_right.as_ptr(),
                  Some(Node(_,NodeType::Macro(_))) => const_cstr!("O").as_ptr(),
                  _ => const_cstr!("?").as_ptr(),
              };

              lowest = lowest.min(v.1);
              let selected = if let Selection::Entity(EntityId::Node(id)) = &app.model.view.selection { id == k } else { false };
              let hover = igIsMouseHoveringRect(tl,br,false);
              ImDrawList_AddText(draw_list, tl, 
                                 if selected { selected_col } 
                                 else if canvas_hovered && hover { line_hover_col } else { line_col }, 
                                 symbol, ptr::null());
              if hover {
                  hovered_item = Some(EntityId::Node(*k));
              }
          }

          // TODO symbol locations are supposed to be stored in the schematic
          // object, not recalculated from Pos
          for (i,Object(track_id, pos, obj)) in app.model.inf.iter_objects() {
              if let Some((loc,tangent)) = s.track_line_at(track_id, *pos) {
                  let rightside = (tangent.1, -tangent.0);
                  match obj {
                      ObjectType::Sight { .. } => {}, // ignore for now
                      ObjectType::Signal(Dir::Up) => {
                          let pw = (loc.0 + rightside.0*0.2, loc.1 + rightside.1*0.2);
                          let ps = world2screen(canvas_pos, canvas_lower, center, zoom, pw);
                          let hovered = dist2(&mouse_pos, &ps) < 100.0;
                          if hovered {
                              hovered_item = Some(EntityId::Object(i));
                          }
                          let selected = if let Selection::Entity(EntityId::Object(id)) = &app.model.view.selection { id == &i } else { false };
                          let color = if selected { selected_col } 
                             else if canvas_hovered && hovered { line_hover_col } else { line_col };
                          ImDrawList_AddText(draw_list, ps, 
                                             color,
                                             const_cstr!("\u{f637}").as_ptr(), ptr::null());
                      },
                      ObjectType::Signal(Dir::Down) => {
                          let pw = (loc.0 - rightside.0*0.2, loc.1 - rightside.1*0.2);
                          let ps = world2screen(canvas_pos, canvas_lower, center, zoom, pw);
                          let hovered = dist2(&mouse_pos, &ps) < 100.0;
                          if hovered {
                              hovered_item = Some(EntityId::Object(i));
                          }
                          let selected = if let Selection::Entity(EntityId::Object(id)) = &app.model.view.selection { id == &i } else { false };
                          let color = if selected { selected_col } 
                             else if canvas_hovered && hovered { line_hover_col } else { line_col };
                          ImDrawList_AddText(draw_list, ps, 
                                             color,
                                             const_cstr!("\u{f637}").as_ptr(), ptr::null());
                      },
                      ObjectType::Balise(filled) => {
                          let pw = (loc.0, loc.1);
                          let ps = world2screen(canvas_pos, canvas_lower, center, zoom, pw);
                          let hovered = dist2(&mouse_pos, &ps) < 100.0;
                          if hovered {
                              hovered_item = Some(EntityId::Object(i));
                          }
                          let selected = if let Selection::Entity(EntityId::Object(id)) = &app.model.view.selection { id == &i } else { false };
                          let color = if selected { selected_col } 
                             else if canvas_hovered && hovered { line_hover_col } else { line_col };
                          ImDrawList_AddText(draw_list, ps, 
                                             color,
                                             const_cstr!("\u{f071}").as_ptr(), ptr::null());
                      },
                      ObjectType::Detector => {
                          let pw1 = (loc.0 - rightside.0*0.1, loc.1 - rightside.1*0.1);
                          let pw2 = (loc.0 + rightside.0*0.1, loc.1 + rightside.1*0.1);
                          let ps1 = world2screen(canvas_pos, canvas_lower, center, zoom, pw1);
                          let ps2 = world2screen(canvas_pos, canvas_lower, center, zoom, pw2);
                          let hovered = dist2(&mouse_pos, &line_closest_pt(&ps1, &ps2, &mouse_pos)) < 100.0;
                          if hovered {
                              hovered_item = Some(EntityId::Object(i));
                          }
                          let selected = if let Selection::Entity(EntityId::Object(id)) = &app.model.view.selection { id == &i } else { false };
                          let color = if selected { selected_col } 
                             else if canvas_hovered && hovered { line_hover_col } else { line_col };
                          ImDrawList_AddLine(draw_list, ps1,ps2, color, 2.0);
                      },
                  }
              }
          }


          // Get instant (occupied TVDs, trains, switch positions, etc.)
          //
          let historygraph = match app.model.view.selected_scenario {
              SelectedScenario::Dispatch(d) => {
                  if let Some(Scenario::Dispatch(Dispatch { history: Derive::Ok(h), .. }))
                      = app.model.scenarios.get_mut(d) { Some(h) } else { None }
              },
              SelectedScenario::Usage(u,Some(d)) => {
                  if let Some(Scenario::Usage(_, Derive::Ok(dispatches)))
                      = app.model.scenarios.get_mut(d) {
                          if let Some(Dispatch { history: Derive::Ok(h), .. })
                              = dispatches.get_mut(d) { Some(h) } else { None }
                      } else { None }
              },
              _ => None,
          };


          if let Some(hg) = historygraph {
              let graph = hg.graph(&app.model.inf, &app.model.dgraph.get().unwrap(), &app.model.schematic.get().unwrap());
              for (g,_) in &graph.instant.geom {
                  match g {
                      DispatchCanvasGeom::SectionStatus(p1,p2,status) => {
                          let color = match status {
                              SectionStatus::Free => line_col,
                              SectionStatus::Reserved => reserved_col,
                              SectionStatus::Occupied => occupied_col,
                              SectionStatus::Overlap => overlap_col,
                          };
                          let ps1 = world2screen(canvas_pos, canvas_lower, center, zoom, *p1);
                          let ps2 = world2screen(canvas_pos, canvas_lower, center, zoom, *p2);
                          ImDrawList_AddLine(draw_list, ps1,ps2, color, 5.0);
                      },
                      DispatchCanvasGeom::SignalAspect(p,object_id, aspect) => {
                      },
                      DispatchCanvasGeom::TrainLoc(p1,p2,id) => {
                          let ps1 = world2screen(canvas_pos, canvas_lower, center, zoom, *p1);
                          let ps2 = world2screen(canvas_pos, canvas_lower, center, zoom, *p2);
                          ImDrawList_AddLine(draw_list, ps1,ps2, selected_col, 10.0);
                      },

                      DispatchCanvasGeom::SwitchStatus(pt,object_id,status) => {
                      }
                  }
              }
          }



          let (mut last_x,mut line_no) = (None,0);
          for (x,_id,pos) in &s.pos_map {
              let x = *x;
              // TODO use line_no to calculate number of text heights to lower
              //println!("{:?}", lowest);
              ImDrawList_AddLine(draw_list,
                                 world2screen(canvas_pos, canvas_lower, center, zoom, (x, lowest - 0.5)),
                                 world2screen(canvas_pos, canvas_lower, center, zoom, (x, lowest - 0.5 - (line_no+1) as f32)),
                                 line_col, 1.0);
              if Some(x) == last_x {
                  line_no += 1;
              } else {
                  line_no = 0;
              }
              let s = CString::new(format!(" {:.3}", pos)).unwrap();
              ImDrawList_AddText(draw_list, 
                                 world2screen(canvas_pos, canvas_lower, center, zoom, (x, lowest - 0.5 - (line_no) as f32)),
                                 line_col,
                                 s.as_ptr(), ptr::null());
              last_x = Some(x);
          }

          // highlight ruote
          if let Some(route_idx) = &app.model.view.hot_route {
              if let Derive::Ok(routes) = &app.model.interlocking.routes {
                if let Some(route) = routes.0.get(*route_idx) {
                    // Draw start signal / boundary green
                    // Draw end signal/boundary red
                    // Draw switch positions
                    // Draw sections
                    // ... and color release groups differently?
                }
              }
          }

          if let Selection::Pos(pos, y, id) = &app.model.view.selection {
              if let Some(x) = s.find_pos(*pos) {
                  //println!("Drawing at {:?} {:?}", x, y);
                ImDrawList_AddLine(draw_list, 
                   world2screen(canvas_pos, canvas_lower, center, zoom, (x, y - 0.25)),
                   world2screen(canvas_pos, canvas_lower, center, zoom, (x, y + 0.25)),
                   selected_col, 2.0);
              }
          }

          ImDrawList_PopClipRect(draw_list);

          if clicked {
              app.clicked_object(hovered_item, 
                                 screen2world(canvas_pos, canvas_lower, center, zoom, (*io).MousePos));
          }

          if right_clicked {
              if let Some(id) = hovered_item {
                  igOpenPopup(const_cstr!("canvasctx").as_ptr());
                  app.model.view.canvas_context_menu_item = Some(id);
              } else {
                if let Some(screen) = app.context_menu() {
                    app.command_builder = Some(CommandBuilder::new_screen(screen));
                }
              }
              
          }

          let mut cmd = None;
          if igBeginPopup(const_cstr!("canvasctx").as_ptr(),0) {
              match app.model.view.canvas_context_menu_item {
                  Some(EntityId::Node(node_id)) => {
                      if let Some(Node(_, NodeType::Macro(_))) = app.model.inf.get_node(&node_id) {
                          show_text("Boundary node");
                          igSeparator();
                          if let SelectedScenario::Dispatch(disp_id) = &app.model.view.selected_scenario {
                              if let Some(d) = app.model.dgraph.get() {
                                  for (ri,r) in app.model.interlocking.routes_from_boundary(d,node_id) {
                                      igPushIDInt(ri as _);

                                      let train_to = CString::new(format!("Train to {:?} ({})", r.exit, ri)).unwrap();
                                      if igBeginMenu(train_to.as_ptr(), true) {
                                          for (vi,vehicle) in app.model.vehicles.iter().enumerate() {
                                              igPushIDInt(vi as _);
                                              let vname = CString::new(vehicle.name.clone()).unwrap();
                                              if igMenuItemBool(vname.as_ptr(), ptr::null(), false, true) {
                                                cmd = Some(ScenarioEdit::AddDispatchCommand(*disp_id, 
                                                                                            0.0,
                                                            // TODO current instant
                                                            //app.model.view.instant.as_ref().map(|i| i.time as f32).unwrap_or(0.0 as f32), 
                                                            Command::Train(vi,ri))); // TODO i is wrong  (??)
                                              }
                                              igPopID();
                                          }
                                          igEndMenu();
                                      }

                                      igPopID();
                                  }
                              }
                          }
                      }
                  },
                  Some(_) | None => {
                      show_text("Nothing to see here.");
                  },
              }
              igEndPopup();
          }

          if let Some(cmd) = cmd {
              app.integrate(AppAction::Model(ModelAction::Scenario(cmd)));
          }


          if let Some(id) = hovered_item {
              if canvas_hovered {
                  igBeginTooltip();
                  // TODO entity_to_string
                  //show_text(&entity_to_string(id, &app.model.inf));
                  show_text("entity_to_string here");
                  igEndTooltip();
              }
          }

      },
  }

  igEndChild();

  capture_canvas_key
    }
}
