use backend_glfw::imgui::*;
use const_cstr::*;
use crate::gui::widgets;
use crate::app;
use crate::document::dgraph::*;
use crate::gui::widgets::Draw;
use crate::document::infview::InfView;
use crate::config::*;
use crate::util;
use crate::document::model::PtC;
use nalgebra_glm as glm;

pub fn debug_window(mut popen :bool, app :&app::App, inf_canvas :Option<&Draw>, inf_view :&InfView) -> bool {
    if !popen { return popen; }
    unsafe {
    widgets::next_window_center_when_appearing();
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

        let model = app.document.analysis.model();

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
        let derived = app.document.analysis.data();
        if igTreeNodeStr(const_cstr!("Topology").as_ptr()) {
            widgets::show_text(&format!("{:#?}", derived.topology));
            igTreePop();
        }
        if igTreeNodeStr(const_cstr!("DGraph").as_ptr()) {
            widgets::show_text(&format!("{:#?}", derived.dgraph));
            igTreePop();
        }
        if igTreeNodeStr(const_cstr!("All paths").as_ptr()) {
            if let Some((_,dgraph)) = &derived.dgraph {
                let (l,paths) = &dgraph.all_paths;
                widgets::show_text(&format!("{} paths, with equality length margin {:.3}m", paths.len(), l));
                for (i,p) in paths.iter().enumerate() {
                    igPushIDInt(i as _);
                    show_path(&app.config, dgraph, p, inf_canvas, inf_view);
                    igPopID();
                }
            }
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

fn show_path(config :&Config, dgraph :&DGraph, path :&[allpaths::Edge], 
             inf_canvas :Option<&Draw>, inf_view :&InfView) {
    unsafe {
        igSelectable(const_cstr!("").as_ptr(), false, 0 as _, ImVec2::zero());
        if igIsItemHovered(0) {
            if let Some(inf_canvas) = inf_canvas {
                let text_color = 255 << 24;
                let color = config.color_u32(RailUIColorName::CanvasRouteSection);
                for (a,b,_l) in path {
                    if let Some((v,forward)) = util::get_symm(&dgraph.edge_lines, (*a,*b)) {
                        draw_lines(color, inf_canvas, inf_view, v);
                        if v.len() >= 2 {
                            let a = v.iter().next().unwrap();
                            let b = v.last().unwrap();
                            if a.x == b.x { continue; }
                            let mid = glm::lerp(a,b,0.5);
                            let symbol = if (a.x < b.x) ^ !forward { 
                                    const_cstr!("\u{f061}")
                                } else { const_cstr!("\u{f060}") };

                            ImDrawList_AddText(inf_canvas.draw_list,
                                               inf_canvas.pos + inf_view.view.world_ptc_to_screen(mid),
                                               text_color,
                                               symbol.as_ptr(),
                                               symbol.as_ptr().offset(symbol.to_bytes().len() as isize));
                        }
                    }
                }
            }
        }
        igSameLine(0.0,-1.0);
        widgets::show_text(&format!("Path with {} segments, length {:.3}m.", 
                                    path.len(), allpaths::path_length(&path)));
    }
}

fn draw_lines(color :u32, canvas :&Draw, view :&InfView, v :&[PtC]) {
    unsafe {
        for (pt_a,pt_b) in v.iter().zip(v.iter().skip(1)) {
            ImDrawList_AddLine(canvas.draw_list,
                               canvas.pos + view.view.world_ptc_to_screen(*pt_a),
                               canvas.pos + view.view.world_ptc_to_screen(*pt_b),
                               color, 14.0);
        }
    }
}
