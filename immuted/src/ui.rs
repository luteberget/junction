use backend_glfw::imgui::*;
pub use backend_glfw::imgui::ImVec2;
use const_cstr::const_cstr;
use std::ptr;


pub mod col {
    use super::*;
    pub fn background() -> u32 {
        unsafe { igGetColorU32Vec4(ImVec4 { x: 0.0, y: 0.0, z: 0.0, w: 1.0  }) }
    }
    pub fn selected() -> u32 {
        unsafe { igGetColorU32Vec4(ImVec4 { x: 0.5, y: 0.5, z: 1.0, w: 1.0  }) }
    }
    pub fn unselected() -> u32 {
        unsafe { igGetColorU32Vec4(ImVec4 { x: 0.95, y: 0.95, z: 1.0, w: 1.0  }) }
    }
    pub fn gridpoint() -> u32 {
        unsafe { igGetColorU32Vec4(ImVec4 { x: 1.0, y: 1.0, z: 1.0, w: 0.25  }) }
    }
    pub fn error() -> u32 {
        unsafe { igGetColorU32Vec4(ImVec4 { x: 1.00, y: 0.1, z: 0.1, w: 1.0  }) }
    }
}

pub fn in_root_window(f :impl FnOnce()) {
    unsafe{
        let zero = ImVec2 { x: 0.0, y: 0.0 };
        let io = igGetIO();
        igSetNextWindowPos(zero, ImGuiCond__ImGuiCond_Always as _ , zero);
        igSetNextWindowSize((*io).DisplaySize, ImGuiCond__ImGuiCond_Always as _);
        let win_flags = ImGuiWindowFlags__ImGuiWindowFlags_NoTitleBar
            | ImGuiWindowFlags__ImGuiWindowFlags_NoCollapse
            | ImGuiWindowFlags__ImGuiWindowFlags_NoResize
            | ImGuiWindowFlags__ImGuiWindowFlags_NoMove
            | ImGuiWindowFlags__ImGuiWindowFlags_NoBringToFrontOnFocus
            | ImGuiWindowFlags__ImGuiWindowFlags_NoNavFocus;
        igBegin(const_cstr!("root").as_ptr(), ptr::null_mut(), win_flags as _);
        f();
        igEnd();
    }
}

pub fn canvas(size :ImVec2, name :*const i8, f :impl FnOnce(*mut ImDrawList,ImVec2) -> Option<()>) {
    unsafe {
        let pos :ImVec2 = igGetCursorScreenPos_nonUDT2().into();
        let draw_list = igGetWindowDrawList();
        ImDrawList_AddRectFilled(draw_list, pos, pos + size, col::background(), 0.0, 0);
        let clicked = igInvisibleButton(name, size);
        ImDrawList_PushClipRect(draw_list, pos, pos+size, true);
        f(draw_list, pos);
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


