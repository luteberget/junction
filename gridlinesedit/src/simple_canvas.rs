use imgui_sys_bindgen::sys::*;
use const_cstr::const_cstr;
use crate::pt::*;

pub struct SimpleCanvas {
    lines :Vec<(Pt,Pt)>,
    adding_line :Option<Pt>,
}

impl SimpleCanvas {
    pub fn new() -> Self { SimpleCanvas {
        lines: Vec::new(),
        adding_line: None,
    }}
}


pub fn simple_canvas(size :&ImVec2, canvas :&mut SimpleCanvas) {
    unsafe {
        let io = igGetIO();
        let draw_list = igGetWindowDrawList();
        let pos = igGetCursorScreenPos_nonUDT2();
        let pos = ImVec2 { x: pos.x, y: pos.y };

        let c1 = igGetColorU32Vec4(ImVec4 { x: 0.0, y: 0.0, z: 0.0, w: 1.0 } );
        let c2 = igGetColorU32Vec4(ImVec4 { x: 1.0, y: 1.0, z: 0.0, w: 1.0 } );
        let c3 = igGetColorU32Vec4(ImVec4 { x: 1.0, y: 0.0, z: 1.0, w: 1.0 } );

        ImDrawList_AddRectFilled(draw_list, 
                                pos, ImVec2 { x: pos.x + size.x, y: pos.y + size.y },
                                c1, 0.0, 0);
        igInvisibleButton(const_cstr!("canvas").as_ptr(), *size);

        let pointer = (*io).MousePos;
        let pointer_incanvas = ImVec2 { x: pointer.x - pos.x, y: pointer.y - pos.y };

        let draw = |c :ImU32, p1 : &ImVec2,p2 : &ImVec2| {
            ImDrawList_AddLine(draw_list,
                               ImVec2 { x: pos.x + p1.x, y: pos.y + p1.y },
                               ImVec2 { x: pos.x + p2.x, y: pos.y + p2.y },
                               c, 2.0);
        };

        match (igIsItemHovered(0), igIsMouseDown(0), &mut canvas.adding_line) {
            (true, true, None) => {
                canvas.adding_line = Some(Pt { x: pointer_incanvas.x, y: pointer_incanvas.y });
            },
            (_,false,Some(pt)) => {
                canvas.lines.push((*pt, Pt { x: pointer_incanvas.x, y: pointer_incanvas.y }));
                canvas.adding_line = None;
            },
            _ => {},
        }

        // clip
        ImDrawList_PushClipRect(draw_list, pos, ImVec2 { x: pos.x + size.x, y: pos.y + size.y}, true);

        // Draw permanent lines
        //
        for (p1,p2) in &canvas.lines {
            draw(c2, &ImVec2 { x: p1.x, y: p1.y },
                 &ImVec2 { x: p2.x, y: p2.y });
        }

        // Draw temporary line
        if let Some(pt) = &canvas.adding_line {
            draw(c3, &ImVec2 { x: pt.x, y: pt.y }, &pointer_incanvas);
        }


        // pop clip
        ImDrawList_PopClipRect(draw_list);
    }
}
