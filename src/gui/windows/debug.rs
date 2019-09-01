use backend_glfw::imgui::*;
use const_cstr::*;
use crate::gui::widgets;
use crate::app;

pub fn debug_window(mut popen :bool, app :&app::App) -> bool {
    if !popen { return popen; }
    unsafe {
    igBegin(const_cstr!("View data").as_ptr(), &mut popen as _, 0 as _);
    igPushTextWrapPos(0.0);

    let defaultopen = ImGuiTreeNodeFlags__ImGuiTreeNodeFlags_DefaultOpen;

    if igTreeNodeExStr(const_cstr!("Application state").as_ptr(), defaultopen as _) {
        widgets::show_text(&format!("{:#?}", app.document.fileinfo));
        //ui::show_text(&app.document.viewmodel.info());

        //if igTreeNodeStr(const_cstr!("Canvas").as_ptr()) {
        //    ui::show_text(&format!("{:#?}", allstate.canvas));
        //    igTreePop();
        //}
        //if igTreeNodeStr(const_cstr!("Diagram").as_ptr()) {
        //    ui::show_text(&format!("{:#?}", allstate.diagram));
        //    igTreePop();
        //}
        igTreePop();
    }


    if igTreeNodeStr(const_cstr!("Model").as_ptr()) {
        // TODO threads 
        //ui::show_text(&allstate.viewmodel.get_undoable().info());

        let model = app.document.viewmodel.model();

        if igTreeNodeStr(const_cstr!("Line segments").as_ptr()) {
            widgets::show_text(&format!("{:#?}", model.linesegs));
            igTreePop();
        }
        if igTreeNodeStr(const_cstr!("Objects").as_ptr()) {
            widgets::show_text(&format!("{:#?}", model.objects));
            igTreePop();
        }
        if igTreeNodeStr(const_cstr!("Node data override").as_ptr()) {
            widgets::show_text(&format!("{:#?}", model.node_data));
            igTreePop();
        }
        if igTreeNodeStr(const_cstr!("Vehicles").as_ptr()) {
            widgets::show_text(&format!("{:#?}", model.vehicles));
            igTreePop();
        }
        if igTreeNodeStr(const_cstr!("Dispatches").as_ptr()) {
            widgets::show_text(&format!("{:#?}", model.dispatches));
            igTreePop();
        }
        if igTreeNodeStr(const_cstr!("Plans").as_ptr()) {
            widgets::show_text(&format!("{:#?}", model.plans));
            igTreePop();
        }
        igTreePop();
    }

    if igTreeNodeStr(const_cstr!("Derived data / view model").as_ptr()) {
        let derived = app.document.viewmodel.data();
        if igTreeNodeStr(const_cstr!("Topology").as_ptr()) {
            widgets::show_text(&format!("{:#?}", derived.topology));
            igTreePop();
        }
        if igTreeNodeStr(const_cstr!("DGraph").as_ptr()) {
            widgets::show_text(&format!("{:#?}", derived.dgraph));
            igTreePop();
        }
        if igTreeNodeStr(const_cstr!("Interlocking").as_ptr()) {
            widgets::show_text(&format!("{:#?}", derived.interlocking));
            igTreePop();
        }
        if igTreeNodeStr(const_cstr!("Dispatch").as_ptr()) {
            widgets::show_text(&format!("{:#?}", derived.dispatch));
            igTreePop();
        }
        igTreePop();
    }



    igPopTextWrapPos();
    igEnd();
    }

    popen
}

