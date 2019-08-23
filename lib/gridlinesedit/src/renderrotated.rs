use imgui_sys_bindgen::sys::*;
//use const_cstr::const_cstr;

//
//
// TODO WIP
//
pub fn render_rotated_char_center(font :*mut ImFont, draw_list :*mut ImDrawList, angle :f32,
                           pos :ImVec2, col :ImU32, c :ImWchar) {
    if c == ' ' as ImWchar  { return; }
    if c == '\t' as ImWchar  { return; }
    if c == '\n' as ImWchar  { return; }
    if c == '\r' as ImWchar  { return; }

    unsafe {
        let glyph :*const ImFontGlyph = ImFont_FindGlyph(font, c);
        if glyph == std::ptr::null() { return; }
        let pos = ImVec2 { x: pos.x + (*font).DisplayOffset.x,
                           y: pos.y + (*font).DisplayOffset.y };

        //let center = ImVec2 { x: 

        ImDrawList_PrimReserve(draw_list, 6, 4);
        ImDrawList_PrimRectUV(draw_list,
            ImVec2 { x: pos.x + (*glyph).X0, y: pos.y + (*glyph).Y0 },
            ImVec2 { x: pos.x + (*glyph).X1, y: pos.y + (*glyph).Y1 },
            ImVec2 { x: (*glyph).U0, y: (*glyph).V0 },
            ImVec2 { x: (*glyph).U1, y: (*glyph).V1 },
            col);
    }
}
