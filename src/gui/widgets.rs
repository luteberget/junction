use backend_glfw::imgui::*;
use const_cstr::const_cstr;
use std::ptr;

use crate::document::model::{PtC, Pt};
use crate::document::view::View;


pub fn in_root_window(f :impl FnOnce()) {
    unsafe{
        let zero = ImVec2 { x: 0.0, y: 0.0 };
        let io = igGetIO();
        igSetNextWindowPos(zero, ImGuiCond__ImGuiCond_Always as _ , zero);
        igSetNextWindowSize((*io).DisplaySize, ImGuiCond__ImGuiCond_Always as _);
        igPushStyleVarFloat(ImGuiStyleVar__ImGuiStyleVar_WindowRounding as _, 0.0);
        let win_flags = ImGuiWindowFlags__ImGuiWindowFlags_NoTitleBar
            | ImGuiWindowFlags__ImGuiWindowFlags_NoCollapse
            | ImGuiWindowFlags__ImGuiWindowFlags_NoResize
            | ImGuiWindowFlags__ImGuiWindowFlags_NoMove
            | ImGuiWindowFlags__ImGuiWindowFlags_NoBringToFrontOnFocus
            | ImGuiWindowFlags__ImGuiWindowFlags_NoNavFocus
            | ImGuiWindowFlags__ImGuiWindowFlags_MenuBar;
        igBegin(const_cstr!("root").as_ptr(), ptr::null_mut(), win_flags as _);
        f();
        igEnd();
        igPopStyleVar(1);
    }
}

pub struct Draw {
    pub draw_list :*mut ImDrawList,
    pub pos :ImVec2,
    pub size :ImVec2,
    pub view :View,
    pub pointer :PtC,
    pub pointer_grid :Pt,
}

pub struct CanvasDraw {
    draw_list :*mut ImDrawList,
    pos :ImVec2,
    size :ImVec2,
    view :View,
}

impl CanvasDraw {
    fn conv(&self, a :PtC) -> ImVec2 {
        self.pos + self.view.world_ptc_to_screen(a)
    }

    pub fn line(&mut self, a :PtC, b :PtC, color :u32, width: f32) {
        unsafe { ImDrawList_AddLine(self.draw_list, self.conv(a), self.conv(b), color, width); }
    }
}

pub struct Canvas {
    name :*const i8,
    bg_color: Option<u32>,
    size: Option<ImVec2>,
    view: View,
}

impl Canvas {
    pub fn new(name :*const i8, view :View) -> Canvas {
        Canvas {
            name: name,
            bg_color: None,
            size: None,
            view: view,
        }
    }

    pub fn size(self, size :ImVec2) -> Canvas { Canvas { size: Some(size), .. self } }
    pub fn stretch(self) -> Canvas { Canvas { size: None, .. self } }
    pub fn background(self, color :u32) -> Canvas { Canvas { bg_color: Some(color), .. self } }

    pub fn draw(self, f:impl FnOnce(&CanvasDraw)) {
        unsafe  {
            let size = self.size.unwrap_or_else(|| igGetContentRegionAvail_nonUDT2().into() );
            let pos :ImVec2 = igGetCursorScreenPos_nonUDT2().into();

            let draw_list = igGetWindowDrawList();
            if let Some(bg)  =self.bg_color {
                ImDrawList_AddRectFilled(draw_list, pos, pos + size, bg, 0.0, 0);
            }
            let _clicked = igInvisibleButton(self.name, size); // TODO do we need this?
            ImDrawList_PushClipRect(draw_list, pos, pos+size, true);

            f(&CanvasDraw { draw_list, pos, size, view: self.view });
            ImDrawList_PopClipRect(draw_list);
        }
    }
}

pub fn canvas(size :ImVec2, color :u32, name :*const i8, view :View, f :impl FnOnce(&mut Draw) -> Option<()>) {
    unsafe {
        let pos :ImVec2 = igGetCursorScreenPos_nonUDT2().into();
        let draw_list = igGetWindowDrawList();
        ImDrawList_AddRectFilled(draw_list, pos, pos + size, color, 0.0, 0);
        let clicked = igInvisibleButton(name, size);
        ImDrawList_PushClipRect(draw_list, pos, pos+size, true);
        let mouse = (*igGetIO()).MousePos - pos;
        let pointer = view.screen_to_world_ptc(mouse);
        let pointer_grid = view.screen_to_world_pt(mouse);
        f(&mut Draw { pos, size, draw_list, view,pointer,pointer_grid });
        ImDrawList_PopClipRect(draw_list);
    }
}

pub fn show_text(s :&str) {
    unsafe {
    igTextSlice(s.as_ptr() as _ , s.as_ptr().offset(s.len() as _ ) as _ );
    }
}

pub struct Splitter {
    horiz :bool,
    before: f32,
    after: f32,
    size: f32,
}

impl Splitter {
    pub fn new(is_horizontal: bool, sz :&mut f32) -> Self {
        unsafe {
            let root = igGetContentRegionAvail_nonUDT2();
            let (same,other) = if is_horizontal { (root.x, root.y) } else { (root.y, root.x) };
            if *sz + 100.0 > same { *sz = same - 100.0 ; }
            let mut after_size = same - *sz;
            igSplitter(is_horizontal, 4.0, sz, &mut after_size, 100.0, 100.0, -1.0);
            Splitter { horiz: is_horizontal, before: *sz, after: after_size, size: other }
        }
    }

    pub fn horizontal(sz1 :&mut f32) -> Self {
        Self::new(true, sz1)
    }

    pub fn vertical(sz1 :&mut f32) -> Self {
        Self::new(false, sz1)
    }

    pub fn left(self, name :*const i8, f :impl FnOnce()) -> Self {
        unsafe {
        igBeginChild(name, ImVec2 {x: if self.horiz { self.before } else { self.size }, 
            y: if self.horiz { self.size } else { self.before } }, false, 0);
        f();
        igEndChild();
        self
        }
    }

    pub fn right(self, name :*const i8, f :impl FnOnce()) {
        unsafe {
        if self.horiz { igSameLine(0.0,-1.0); }
        igBeginChild(name, ImVec2 {x: if self.horiz { self.after } else { self.size }, 
            y: if self.horiz { self.size } else { self.after } }, false, 0);
        f();
        igEndChild();
        }
    }
}



pub fn radio_select<'a, T>(choices: &'a [(*const i8, bool, T)]) -> Option<&'a T> {
    let mut choice = None;
    for (name,selected,value) in choices.iter() {
        unsafe {
        if igRadioButtonBool(*name, *selected) { choice = Some(value); }
        }
    }
    choice
}



pub fn sep() {
    unsafe {
        igSpacing();
        igSeparator();
        igSpacing();
    }
}

pub fn long_text(s :&str) {
    unsafe {
        let num_lines = s.lines().count();
        let clipper = ImGuiListClipper_ImGuiListClipper(num_lines as i32, -1.0);
        
        while ImGuiListClipper_Step(clipper) {
            let start_idx = (*clipper).DisplayStart as usize;
            let end_idx = (*clipper).DisplayEnd as usize;
            for l in s.lines().skip(start_idx).take(end_idx - start_idx) {
                show_text(l);
            }
        }

        ImGuiListClipper_End(clipper);
    }
}
