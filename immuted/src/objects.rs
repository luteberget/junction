use crate::ui::*;
use crate::view::*;
use crate::model::*;
use crate::util::*;
use backend_glfw::imgui::*;
use nalgebra_glm as glm;

#[derive(Copy,Clone,Debug)]
pub struct Symbol {
    pub loc :PtC,
    pub tangent :Vc,
    pub shape :Shape,
}

#[derive(Copy,Clone,Debug)]
pub enum Shape {
    Detector, Signal,
}

impl Symbol {
    pub fn move_to(&mut self, model :&Model, pt :PtC) -> Option<()> {
        if let Some((l,(d1,d2))) = model.get_closest_lineseg(pt) {
            let pt_on_line = project_to_line(pt, glm::vec2(l.0.x as _ ,l.0.y as _ ),
                                                 glm::vec2(l.1.x as _ ,l.1.y as _ ));
            self.loc = pt_on_line;
            None
        } else {
            self.loc = pt;
            Some(())
        }
    }

    pub fn draw(&self, pos :ImVec2, view :&View, draw_list :*mut ImDrawList) {
        let c = col::unselected();
        unsafe {
            let p = pos + view.world_ptc_to_screen(self.loc);
            let scale = 5.0;
            // TODO can this be simplified?
            let tangent = ImVec2 { x: scale * self.tangent.x as f32,
                                   y: scale * -self.tangent.y as f32 };
            let normal  = ImVec2 { x: scale * -self.tangent.y as f32,
                                   y: scale * -self.tangent.x as f32 };

            match self.shape {
                Shape::Detector =>  {
                    ImDrawList_AddLine(draw_list, p - normal, p + normal, c, 2.0);
                },
                Shape::Signal => {
                    ImDrawList_AddLine(draw_list, p, p + tangent, c, 2.0);
                    ImDrawList_AddLine(draw_list, p + normal, p - normal, c, 2.0);
                    ImDrawList_AddCircle(draw_list, p + tangent + tangent, scale, c, 8, 2.0);
                },
            }
        }
    }
}
