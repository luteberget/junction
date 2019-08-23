use backend_glfw::imgui::*;
use const_cstr::*;
use crate::ui;
use crate::AllState;

pub fn debug_window(popen :&mut bool, allstate :AllState) {
    unsafe {
    igBegin(const_cstr!("View data").as_ptr(), popen as _, 0 as _);
    igPushTextWrapPos(0.0);

    let defaultopen = ImGuiTreeNodeFlags__ImGuiTreeNodeFlags_DefaultOpen;

    if igTreeNodeExStr(const_cstr!("Application state").as_ptr(), defaultopen as _) {
        ui::show_text(&format!("{:#?}", allstate.viewmodel.fileinfo));
        ui::show_text(&allstate.viewmodel.info());

        if igTreeNodeStr(const_cstr!("Canvas").as_ptr()) {
            ui::show_text(&format!("{:#?}", allstate.canvas));
            igTreePop();
        }
        if igTreeNodeStr(const_cstr!("Diagram").as_ptr()) {
            ui::show_text(&format!("{:#?}", allstate.diagram));
            igTreePop();
        }
        igTreePop();
    }


    if igTreeNodeStr(const_cstr!("Model").as_ptr()) {
        ui::show_text(&allstate.viewmodel.get_undoable().info());

        let model = allstate.viewmodel.get_undoable().get();

        if igTreeNodeStr(const_cstr!("Line segments").as_ptr()) {
            ui::show_text(&format!("{:#?}", model.linesegs));
            igTreePop();
        }
        if igTreeNodeStr(const_cstr!("Objects").as_ptr()) {
            ui::show_text(&format!("{:#?}", model.objects));
            igTreePop();
        }
        if igTreeNodeStr(const_cstr!("Node data override").as_ptr()) {
            ui::show_text(&format!("{:#?}", model.node_data));
            igTreePop();
        }
        if igTreeNodeStr(const_cstr!("Vehicles").as_ptr()) {
            ui::show_text(&format!("{:#?}", model.vehicles));
            igTreePop();
        }
        if igTreeNodeStr(const_cstr!("Dispatches").as_ptr()) {
            ui::show_text(&format!("{:#?}", model.dispatches));
            igTreePop();
        }
        igTreePop();
    }

    if igTreeNodeStr(const_cstr!("Derived data / view model").as_ptr()) {
        let derived = allstate.viewmodel.get_data();
        if igTreeNodeStr(const_cstr!("Topology").as_ptr()) {
            ui::show_text(&format!("{:#?}", derived.topology));
            igTreePop();
        }
        if igTreeNodeStr(const_cstr!("DGraph").as_ptr()) {
            ui::show_text(&format!("{:#?}", derived.dgraph));
            igTreePop();
        }
        if igTreeNodeStr(const_cstr!("Interlocking").as_ptr()) {
            ui::show_text(&format!("{:#?}", derived.interlocking));
            igTreePop();
        }
        if igTreeNodeStr(const_cstr!("Dispatch").as_ptr()) {
            ui::show_text(&format!("{:#?}", derived.dispatch));
            igTreePop();
        }
        igTreePop();
    }



    igPopTextWrapPos();
    igEnd();
    }
}

