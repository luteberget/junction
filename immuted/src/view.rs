use crate::model::{Pt,PtC};
use crate::ui::ImVec2;
use nalgebra_glm as glm;

#[derive(Debug)]
pub struct View {
    scale :usize,
    translation :ImVec2,
}

impl View {
    pub fn default() -> Self {
        View {
            scale: 35,
            translation: ImVec2 { x: 0.0, y: 0.0 },
        }
    }

    pub fn zoom(&mut self, amount :f32) {
        self.scale = (self.scale as f32 + 3.0*amount).max(20.0).min(150.0).round() as _;
    }

    pub fn translate(&mut self, delta :ImVec2) {
        self.translation = self.translation - delta;
    }

    pub fn screen_to_world_ptc(&self, pt :ImVec2) -> PtC {
        let x =  (self.translation.x + pt.x) / self.scale as f32;
        let y = -(self.translation.y + pt.y) / self.scale as f32;
        glm::vec2(x,y)
    }

    /// Converts and rounds a screen coordinate to the nearest point on the integer grid
    pub fn screen_to_world_pt(&self, pt :ImVec2) -> Pt {
        let p = self.screen_to_world_ptc(pt);
        glm::vec2(p.x.round() as _, p.y.round() as _)
    }


    pub fn world_ptc_to_screen(&self, pt :PtC) -> ImVec2 {
        let x = ((self.scale as f32 * pt.x) as f32)  - self.translation.x;
        let y = ((self.scale as f32 * -pt.y) as f32) - self.translation.y;

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
