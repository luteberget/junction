use const_cstr::*;
use matches::*;
use backend_glfw::imgui::*;
use rolling::input::staticinfrastructure as rolling_inf;
use std::ffi::CString;

use crate::app::App;
use crate::document::*;
use crate::document::model::*;
use crate::document::objects::*;
use crate::document::infview::*;
use crate::document::view::*;
use crate::document::interlocking::*;
use crate::gui::widgets;
use crate::gui::plan;
use crate::config::RailUIColorName;



pub fn node_editor(doc :&mut Document, pt :Pt) -> Option<()> {
    let (nd,_tangent) = doc.data().topology.as_ref()?.locations.get(&pt)?;
    unsafe {
    match nd {
        NDType::OpenEnd | NDType::BufferStop => {
            if let Some(new_value) =
                widgets::radio_select(&[(const_cstr!("Open end").as_ptr(), *nd == NDType::OpenEnd, NDType::OpenEnd),
                                   (const_cstr!("Buffer stop").as_ptr(), *nd == NDType::BufferStop, NDType::BufferStop)]) {

                doc.edit_model(|m| {
                    m.node_data.insert(pt, *new_value);
                    None
                });
            }
        },
        NDType::Sw(side) => {
            widgets::show_text(&format!("Switch ({:?})", side));

            // TODO 
            let mut speed = 60.0;
            igInputFloat(const_cstr!("Deviating speed restr.").as_ptr(), &mut speed, 1.0, 10.0,
                         const_cstr!("%.1f").as_ptr(), 0 as _);
        },
        NDType::Crossing(type_) => {
            widgets::show_text(&format!("Crossing ({:?})", type_));
            if let Some(new_value) =
                widgets::radio_select(&[(const_cstr!("Crossover").as_ptr(), *type_ == CrossingType::Crossover, CrossingType::Crossover),
                                   (const_cstr!("Single slip (above)").as_ptr(), *type_ == CrossingType::SingleSlip(Side::Left), CrossingType::SingleSlip(Side::Left)),
                                   (const_cstr!("Single slip (below)").as_ptr(), *type_ == CrossingType::SingleSlip(Side::Right), CrossingType::SingleSlip(Side::Right)),
                                   (const_cstr!("Double slip").as_ptr(), *type_ == CrossingType::DoubleSlip, CrossingType::DoubleSlip)]) {

                doc.edit_model(|m| {
                    m.node_data.insert(pt, NDType::Crossing(*new_value));
                    None
                });
            }

            // TODO 
            let mut speed = 60.0;
            igInputFloat(const_cstr!("Deviating speed restr.").as_ptr(), &mut speed, 1.0, 10.0,
                         const_cstr!("%.1f").as_ptr(), 0 as _);
        }
        _ => {},
    }
    }
    Some(())
}


pub fn object_menu(doc :&mut Document, pta :PtA) -> Option<()> {
    let obj = doc.model().objects.get(&pta)?;

    let mut set_distant = None;
    for f in obj.functions.iter() {
        match f {
            Function::Detector => { widgets::show_text("Detector"); },
            Function::MainSignal { has_distant } => {
                widgets::show_text("Main signal");
                let mut has_distant = *has_distant;
                unsafe {
                    igCheckbox(const_cstr!("Distant signal").as_ptr(), &mut has_distant);
                    if igIsItemEdited() {
                        set_distant = Some(has_distant);
                    }
                }
            }
        }
    }
    if let Some(d) = set_distant {
        doc.edit_model(|new| {
            new.objects.get_mut(&pta).unwrap().functions = vec![Function::MainSignal { has_distant: d }];
            None
        });
    }
    Some(())
}

pub fn route_selector(doc :&mut Document, thing :Ref, preview :&mut Option<usize>) -> Option<Command> {
    let il = doc.data().interlocking.as_ref()?;
    let routes = il.get_routes(thing)?;

    unsafe {

        let have_dispatch = matches!(&doc.dispatch_view, Some(DispatchView::Manual(_)));
        if have_dispatch {
            widgets::show_text("Add dispatch command:");
        } else {
            widgets::show_text("New dispatch:");
        }

        let mut some = false;
        let mut action = None;
        igIndent(14.0);
        for idx in routes {
            some = true;
            igPushIDInt(*idx as _);

            let is_boundary = matches!(il.routes[*idx].route.entry, rolling_inf::RouteEntryExit::Boundary(_));
            let text = CString::new(format!("Route to {:?}", (il.routes[*idx].route).exit)).unwrap();

            if is_boundary {
                if igBeginMenu(text.as_ptr(), true) {
                    if let Some(train_id) = plan::select_train(doc.model()) {
                        action = Some(Command::Train(train_id, il.routes[*idx].id));
                    }
                    *preview = Some(*idx);
                    igEndMenu();
                }
            } else {
                if igSelectable(text.as_ptr(), false, 0 as _, ImVec2::zero()) {
                    action = Some(Command::Route(il.routes[*idx].id));
                }
                if igIsItemHovered(0) {
                    *preview = Some(*idx);
                }
            }
            igPopID();

        }
        if !some {
            widgets::show_text("No routes.");
        }
        igUnindent(14.0);
        action
    }
}


pub fn add_plan_visit(doc :&mut Document, thing :Ref) {

}



