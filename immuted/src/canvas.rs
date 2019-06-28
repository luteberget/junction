use crate::model::*;
use std::collections::HashSet;
use crate::ui;
use crate::ui::ImVec2;
use backend_glfw::imgui::*;

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
        ui::canvas(size, |draw_list, pos| {

            // Hotkeys
            let handle_keys = unsafe { igIsItemActive() || !igIsAnyItemActive() };
            if handle_keys { self.handle_keys(); }

            // Draw background
            self.draw_background(doc.get(), draw_list, pos);

            // Scroll action (wheel or ctrl-drag)
            self.scroll();

            // Edit actions 
            match self.state {
                Action::None =>  {
                }
                Action::DrawingLine(from) => {
                },
                Action::DrawObjectType(objtype) => {
                },
                Action::ContextMenu(ContextMenu::Node(pt)) => {
                },
                Action::ContextMenu(ContextMenu::Track(pa,pb)) => {
                },
                Action::ContextMenu(ContextMenu::Object(p)) => {
                },
            }

        });
    }

    pub fn handle_keys(&mut self) {
    }

    pub fn scroll(&mut self) {
        unsafe {
            let io = igGetIO();
            let wheel = (*io).MouseWheel;
            if wheel != 0.0 {
                self.scale = (self.scale as f32 + 3.0*wheel).max(20.0).min(150.0).round() as _;
            }
        }
    }

    pub fn draw_background(&self, m :&Model, draw_list :*mut ImDrawList, pos :ImVec2) {
        unsafe {
            for l in &m.linesegs {
                let col = if self.selection.tracks.contains(l) { ui::selected() } else { ui::unselected() };
                ImDrawList_AddLine(draw_list, pos + self.world_pt_to_screen(l.0), 
                                              pos + self.world_pt_to_screen(l.1), col, 2.0);
            }
        }
    }

    pub fn world_pt_to_screen(&self, pt :Pt) -> ImVec2 {
        let x = ((self.scale as i32 *  pt.x) as f32) - self.translation.x;
        let y = ((self.scale as i32 * -pt.y) as f32) - self.translation.y;
        ImVec2 { x, y }
    }
}
