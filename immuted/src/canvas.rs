use crate::model::*;
use std::collections::HashSet;
use crate::ui;
use crate::Derived;
use crate::objects::*;
use crate::util;
use crate::view::*;
use crate::ui::col;
use crate::ui::ImVec2;
use backend_glfw::imgui::*;
use nalgebra_glm as glm;
use const_cstr::const_cstr;
use matches::matches;

pub struct Canvas {
    action :Action,
    selection :HashSet<Ref>,
    view :View,
}

#[derive(Debug)]
pub enum Action {
    Normal(NormalState),
    DrawingLine(Option<Pt>),
    InsertObject(Option<Object>),
}

#[derive(Debug,Copy,Clone)]
pub enum NormalState {
    Default,
    SelectWindow(ImVec2),
    DragMove,
}

impl Canvas {
    pub fn new() -> Self {
        Self {
            action :Action::Normal(NormalState::Default),
            selection :HashSet::new(),
            view :View::default(),
        }
    }

    //pub fn toolbar(&mut self, doc :&mut Undoable<Model>) {
    pub fn toolbar(&mut self) { unsafe {
        if tool_button(const_cstr!("select (A)").as_ptr(),
            'A' as _, matches!(&self.action, Action::Normal(_))) {
            self.action = Action::Normal(NormalState::Default);
        }
        igSameLine(0.0,-1.0);
        if tool_button(const_cstr!("insert object (S)").as_ptr(),
            'S' as _, matches!(&self.action, Action::InsertObject(_))) {
            self.action = Action::InsertObject(None);
        }
        igSameLine(0.0,-1.0);
        if tool_button(const_cstr!("draw track (D)").as_ptr(),
            'D' as _, matches!(&self.action, Action::DrawingLine(_))) {
            self.action = Action::DrawingLine(None);
        }
    } }

    pub fn draw(&mut self, doc :&mut Undoable<Model>, derived :&mut Derived) {
        self.toolbar();

        let zero = ImVec2 { x: 0.0, y: 0.0 };
        use backend_glfw::imgui::*;
        let size = unsafe { igGetContentRegionAvail_nonUDT2().into() };
        ui::canvas(size, |draw_list, pos| { unsafe {

            // Hotkeys
            self.handle_global_keys(doc);
            //hotkey!(CTRL+Z, { doc.undo(); });
            let handle_keys = igIsItemActive() || !igIsAnyItemActive();
            if handle_keys { self.handle_keys(); }

            // Scroll action (wheel or ctrl-drag)
            self.scroll();

            let io = igGetIO();
            let pointer = (*io).MousePos - pos;
            let pointer_ongrid = self.view.screen_to_world_pt(pointer);
            let pointer_ingrid = self.view.screen_to_world_ptc(pointer);

            // Context menu 
            if igBeginPopup(const_cstr!("ctx").as_ptr(), 0 as _) {
                if !self.selection.is_empty() {
                    if igSelectable(const_cstr!("Delete").as_ptr(), false, 0 as _, zero) {
                        self.delete_selection(doc);
                    }
                }
                igEndPopup();
            }

            // Edit actions 
            match &mut self.action {
                Action::Normal(normal) => {
                    let normal = *normal;
                    self.normalstate(normal, doc, draw_list, pointer_ingrid, pos);
                }
                Action::DrawingLine(from) => {
                    let from = *from;
                    self.drawingline(doc,from,pos,pointer_ongrid,draw_list);
                }
                Action::InsertObject(None) => {
                },
                Action::InsertObject(Some(obj)) => {
                    let moved = obj.symbol.move_to(doc.get(),pointer_ingrid);
                    obj.symbol.draw(pos,&self.view,draw_list);
                    if let Some(err) = moved {
                        let p = pos + self.view.world_ptc_to_screen(obj.symbol.loc);
                        let window = ImVec2 { x: 4.0, y: 4.0 };
                        ImDrawList_AddRect(draw_list, p - window, p + window, col::error(), 0.0,0,4.0);
                    } else  {
                        if igIsMouseReleased(0) {
                            let mut m = doc.get().clone();
                            m.objects.insert(round_coord(obj.symbol.loc), obj.clone());
                            doc.set(m);
                        }
                    }
                },
            };

            // Draw background
            self.draw_background(doc.get(), draw_list, pos, size);


        }});
    }

    pub fn handle_keys(&mut self) {
        unsafe {
        if igIsKeyPressed('A' as _, false) {
            self.action = Action::Normal(NormalState::Default);
        }
        if igIsKeyPressed('D' as _, false) {
            self.action = Action::DrawingLine(None);
        }
        if igIsKeyPressed('S' as _, false) {
            if let Action::InsertObject(Some(Object { symbol: Symbol { shape: Shape::Detector, .. } } )) = &self.action {
                    self.action = Action::InsertObject(Some(
                            Object { symbol: Symbol { 
                                loc: glm::vec2(0.0,0.0), 
                                tangent :glm::vec2(1,0),
                                shape: Shape::Signal } } ));

            } else {
                    self.action = Action::InsertObject(Some(
                            Object { symbol: Symbol { 
                                loc: glm::vec2(0.0,0.0), 
                                tangent :glm::vec2(1,0),
                                shape: Shape::Detector } } ));
            }
        }
        }
    }

    pub fn handle_global_keys(&mut self, doc :&mut Undoable<Model>) { unsafe {
        let io = igGetIO();
        if (*io).KeyCtrl && !(*io).KeyShift && igIsKeyPressed('Z' as _, false) {
            doc.undo();
        }
        if (*io).KeyCtrl && (*io).KeyShift && igIsKeyPressed('Z' as _, false) {
            doc.redo();
        }
        if (*io).KeyCtrl && !(*io).KeyShift && igIsKeyPressed('Y' as _, false) {
            doc.redo();
        }
    } }

    pub fn scroll(&mut self) {
        unsafe {
            let io = igGetIO();
            let wheel = (*io).MouseWheel;
            if wheel != 0.0 {
                self.view.zoom(wheel);
            }
            if ((*io).KeyCtrl && igIsMouseDragging(0,-1.0)) || igIsMouseDragging(2,-1.0) {
                self.view.translate((*io).MouseDelta);
            }
        }
    }

    pub fn draw_background(&self, m :&Model, draw_list :*mut ImDrawList, pos :ImVec2, size :ImVec2) {
        unsafe {

            let sel_window = if let Action::Normal(NormalState::SelectWindow(a)) = &self.action {
                Some((*a, *a + igGetMouseDragDelta_nonUDT2(0,-1.0).into()))
            } else { None };

            for l in &m.linesegs {
                let p1 = self.view.world_pt_to_screen(l.0);
                let p2 = self.view.world_pt_to_screen(l.1);
                let selected = self.selection.contains(&Ref::Track(l.0,l.1));
                let preview = sel_window
                    .map(|(a,b)| util::point_in_rect(p1,a,b) || util::point_in_rect(p2,a,b))
                    .unwrap_or(false) ;
                let col = if selected || preview { col::selected() } else { col::unselected() };
                ImDrawList_AddLine(draw_list, pos + p1, pos + p2, col, 2.0);
            }

            let (lo,hi) = self.view.points_in_view(size);
            for x in lo.x..=hi.x {
                for y in lo.y..=hi.y {
                    let pt = self.view.world_pt_to_screen(glm::vec2(x,y));
                    ImDrawList_AddCircleFilled(draw_list, pos+pt, 3.0, col::gridpoint(), 4);
                }
            }

            for (_pta,obj) in &m.objects {
                obj.symbol.draw(pos, &self.view, draw_list);
            }
        }
    }

    pub fn set_selection_window(&mut self, doc :&Undoable<Model>, a :ImVec2, b :ImVec2) {
        self.selection = doc.get().get_rect(self.view.screen_to_world_ptc(a),
                                            self.view.screen_to_world_ptc(b))
                        .into_iter().collect();
    }

    pub fn normalstate(&mut self, state: NormalState, doc :&Undoable<Model>,
                       draw_list :*mut ImDrawList, pointer_ingrid :PtC, pos :ImVec2) {
        unsafe {
        let io = igGetIO();
        match state {
            NormalState::SelectWindow(a) => {
                let b = a + igGetMouseDragDelta_nonUDT2(0,-1.0).into();
                if igIsMouseDragging(0,-1.0) {
                    ImDrawList_AddRect(draw_list, pos + a, pos + b,
                                       col::selected(),0.0, 0, 1.0);
                } else {
                    self.set_selection_window(doc, a,b);
                    self.action = Action::Normal(NormalState::Default);
                }
            },
            NormalState::DragMove => {
                if igIsMouseDragging(0,-1.0) {
                    // TODO 
                } else {
                    self.action = Action::Normal(NormalState::Default);
                }
            }
            NormalState::Default => {
                if igIsMouseDragging(0,-1.0) {
                    if let Some((r,_)) = doc.get().get_closest(pointer_ingrid) {
                        if !self.selection.contains(&r) {
                            self.selection = std::iter::once(r).collect();
                        }
                        self.action = Action::Normal(NormalState::DragMove);
                    } else {
                        let a = (*io).MouseClickedPos[0] - pos;
                        //let b = a + igGetMouseDragDelta_nonUDT2(0,-1.0).into();
                        self.action = Action::Normal(NormalState::SelectWindow(a));
                    }
                } else {
                    if igIsItemActive() && igIsMouseReleased(0) {
                        if !(*io).KeyShift { self.selection.clear(); }
                        if let Some((r,_)) = doc.get().get_closest(pointer_ingrid) {
                            self.selection.insert(r);
                        } 
                    }
                    if igIsMouseClicked(1,false) {
                        if let Some((r,_)) = doc.get().get_closest(pointer_ingrid) {
                            if !self.selection.contains(&r) {
                                self.selection = std::iter::once(r).collect();
                            }
                        }
                        igOpenPopup(const_cstr!("ctx").as_ptr());
                    }
                }
            },
        }
        }
    }

    pub fn drawingline(&mut self,  doc :&mut Undoable<Model>,from :Option<Pt>,
                       pos :ImVec2, pointer_ongrid :Pt, draw_list :*mut ImDrawList
                       ) {
        unsafe {
        // Draw preview
        if let Some(pt) = from {
            for (p1,p2) in util::route_line(pt, pointer_ongrid) {
                ImDrawList_AddLine(draw_list, pos + self.view.world_pt_to_screen(p1),
                                              pos + self.view.world_pt_to_screen(p2), 
                                              col::selected(), 2.0);
            }

            if !igIsMouseDown(0) {
                let mut new_model = doc.get().clone();
                for (p1,p2) in util::route_line(pt,pointer_ongrid) {
                    let unit = util::unit_step_diag_line(p1,p2);
                    for (pa,pb) in unit.iter().zip(unit.iter().skip(1)) {
                        new_model.linesegs.insert(util::order_ivec(*pa,*pb));
                    }
                }
                doc.set(new_model);
                self.action = Action::DrawingLine(None);
            }
        } else {
            if igIsItemHovered(0) && igIsMouseDown(0) {
                self.action = Action::DrawingLine(Some(pointer_ongrid));
            }
        }
    } }

    pub fn delete_selection(&mut self, doc :&mut Undoable<Model>) {
        let mut new_model = doc.get().clone();
        for x in self.selection.drain() {
            new_model.delete(x);
        }
        doc.set(new_model);
    }
}


fn tool_button(name :*const i8, char :i8, selected :bool) -> bool {
        unsafe {
        if selected {
            let c1 = ImVec4 { x: 0.4, y: 0.65,  z: 0.4, w: 1.0 };
            let c2 = ImVec4 { x: 0.5, y: 0.85, z: 0.5, w: 1.0 };
            let c3 = ImVec4 { x: 0.6, y: 0.9,  z: 0.6, w: 1.0 };
            igPushStyleColor(ImGuiCol__ImGuiCol_Button as _, c1);
            igPushStyleColor(ImGuiCol__ImGuiCol_ButtonHovered as _, c1);
            igPushStyleColor(ImGuiCol__ImGuiCol_ButtonActive as _, c1);
        }
        let clicked = igButton( name , ImVec2 { x: 0.0, y: 0.0 } );
        if selected {
            igPopStyleColor(3);
        }
        clicked
    }
}

//fn unround_coord
fn round_coord(p :PtC) -> PtA {
    let coeff = 10.0;
    glm::vec2((p.x * coeff) as _, (p.y * coeff) as _)
}
