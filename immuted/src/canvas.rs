use crate::model::*;
use std::collections::HashSet;
use crate::ui;
use crate::util;
use crate::ui::col;
use crate::ui::ImVec2;
use backend_glfw::imgui::*;
use nalgebra_glm as glm;

pub struct Canvas {
    state :Action,
    selection :Selection,
    //dispatch :Option<(usize,f32)>,
    scale :usize,
    translation :ImVec2,
}

pub enum Action {
    None,
    DrawingLine(Option<Pt>),
    DrawObjectType(Option<usize>),
    ContextMenu(ContextMenu),
}

pub enum ContextMenu {
    Node(Pt),
    Track(Pt,Pt),
    Object(PtA),
}

pub struct Selection {
    tracks :HashSet<(Pt,Pt)>,
    objects :HashSet<PtA>,
}

impl Selection {
    pub fn empty() -> Self {
        Self {
            tracks: HashSet::new(),
            objects: HashSet::new(),
        }
    }
}

impl Canvas {
    pub fn new() -> Self {
        Self {
            state :Action::None,
            selection :Selection::empty(),
            //dispatch : None,
            scale: 35,
            translation: ImVec2 { x: 0.0, y: 0.0 },
        }
    }

    pub fn draw(&mut self, doc :&mut Undoable<Model>, size :ImVec2) {
        use backend_glfw::imgui::*;
        ui::canvas(size, |draw_list, pos| { unsafe {

            // Hotkeys
            self.handle_global_keys(doc);
            let handle_keys = igIsItemActive() || !igIsAnyItemActive();
            if handle_keys { self.handle_keys(); }

            // Draw background
            self.draw_background(doc.get(), draw_list, pos, size);

            // Scroll action (wheel or ctrl-drag)
            self.scroll();

            let io = igGetIO();
            let pointer_ongrid = self.screen_to_world_pt((*io).MousePos);

            // Edit actions 
            match self.state {
                Action::None =>  {
                    // leftclick nowhere = unselect
                    // leftclick single = select single
                    // leftdrag outside selection = select window
                    // leftdrag single = drag selection (or single if no selection)
                    // rightclick = contextmenu
                    // delete button
                    self.state = Action::DrawingLine(None);
                }
                Action::DrawingLine(from) => {

                    // Draw preview
                    if let Some(pt) = from {
                        for (p1,p2) in util::route_line(pt, pointer_ongrid) {
                            ImDrawList_AddLine(draw_list, pos + self.world_pt_to_screen(p1),
                                                          pos + self.world_pt_to_screen(p2), 
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
                            self.state = Action::DrawingLine(None);
                        }
                    }

                    if from.is_none() && igIsItemHovered(0) && igIsMouseDown(0) {
                        self.state = Action::DrawingLine(Some(pointer_ongrid));
                    }

                    if igIsMouseClicked(1,false) { self.state = Action::None; }

                },
                Action::DrawObjectType(d) => {
                },
                Action::ContextMenu(ContextMenu::Node(pt)) => {
                },
                Action::ContextMenu(ContextMenu::Track(pa,pb)) => {
                },
                Action::ContextMenu(ContextMenu::Object(p)) => {
                },
            }

        }});
    }

    pub fn handle_keys(&mut self) {
    }

    pub fn handle_global_keys(&mut self, doc :&mut Undoable<Model>) { unsafe {
        let io = igGetIO();
        if (*io).KeyCtrl && !(*io).KeyShift && igIsKeyPressed('Z' as _, false) {
            println!("undo {:?}", doc.undo());
        }
        if (*io).KeyCtrl && (*io).KeyShift && igIsKeyPressed('Z' as _, false) {
            println!("redo {:?}", doc.redo());
        }
    } }

    pub fn scroll(&mut self) {
        unsafe {
            let io = igGetIO();
            let wheel = (*io).MouseWheel;
            if wheel != 0.0 {
                self.scale = (self.scale as f32 + 3.0*wheel).max(20.0).min(150.0).round() as _;
            }
            if (*io).KeyCtrl && igIsMouseDragging(0,-1.0) {
                self.translation.x -= (*io).MouseDelta.x;
                self.translation.y -= (*io).MouseDelta.y;
            }
        }
    }

    pub fn draw_background(&self, m :&Model, draw_list :*mut ImDrawList, pos :ImVec2, size :ImVec2) {
        unsafe {
            for l in &m.linesegs {
                let col = if self.selection.tracks.contains(l) { col::selected() } else { col::unselected() };
                ImDrawList_AddLine(draw_list, pos + self.world_pt_to_screen(l.0), 
                                              pos + self.world_pt_to_screen(l.1), col, 2.0);
            }

            let (lo,hi) = self.points_in_view(size);
            for x in lo.x..=hi.x {
                for y in lo.y..=hi.y {
                    let pt = self.world_pt_to_screen(glm::vec2(x,y));
                    ImDrawList_AddCircleFilled(draw_list, pos+pt, 3.0, col::gridpoint(), 4);
                }
            }
        }
    }

    pub fn screen_to_world_ptc(&self, pt :ImVec2) -> (f32,f32) {
        let x =  (self.translation.x + pt.x) / self.scale as f32;
        let y = -(self.translation.y + pt.y) / self.scale as f32;
        (x,y)
    }

    /// Converts and rounds a screen coordinate to the nearest point on the integer grid
    pub fn screen_to_world_pt(&self, pt :ImVec2) -> Pt {
        let (x,y) = self.screen_to_world_ptc(pt);
        glm::vec2(x.round() as _, y.round() as _)
    }


    pub fn world_ptc_to_screen(&self, pt :(f32,f32)) -> ImVec2 {
        let x = ((self.scale as f32 * pt.0) as f32)  - self.translation.x;
        let y = ((self.scale as f32 * -pt.1) as f32) - self.translation.y;

        ImVec2 { x, y }
    }

    /// Convert a point on the integer grid into screen coordinates
    pub fn world_pt_to_screen(&self, pt :Pt) -> ImVec2 {
        let x = ((self.scale as i32 * pt.x) as f32)  - self.translation.x;
        let y = ((self.scale as i32 * -pt.y) as f32) - self.translation.y;

        ImVec2 { x, y }
    }

    /// Return the rect of grid points within the current view.
    pub fn points_in_view(&self, size :ImVec2) -> (Pt,Pt) {
        let lo = self.screen_to_world_pt(ImVec2 { x: 0.0, y: size.y });
        let hi = self.screen_to_world_pt(ImVec2 { x: size.x, y: 0.0 });
        (lo,hi)
    }

}
