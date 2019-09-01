use crate::document::Document;
use crate::document::model::*;
use const_cstr::*;
use backend_glfw::imgui::*;
use crate::gui::widgets;

pub fn edit_vehicles(doc :&mut Document) {
    unsafe {
    let mut new_model = doc.viewmodel.model().clone();
    let mut modified = None;
    for (i,v) in doc.viewmodel.model().vehicles.iter() {
        igPushIDInt(*i as _);

        let mut name = v.name.clone().into_bytes();
        for i in 0..3 { name.push('#' as _); } 
        name.push(0);
        if igCollapsingHeader(name.as_ptr() as _, 0) {
            for i in 0..(3+1) { name.pop(); }
            name.extend((0..15).map(|_| 0));
            igInputText(const_cstr!("Name").as_ptr(),
                name.as_ptr() as *mut _, 
                name.len(),
                0 as _, None, std::ptr::null_mut());

            if igIsItemEdited() {
                let terminator = name.iter().position(|&c| c == 0).unwrap();
                name.truncate(terminator);
                let s = String::from_utf8_unchecked(name);
                new_model.vehicles.get_mut(*i).unwrap().name = s;
                modified = Some(EditClass::VehicleName(*i));
            }

            let format = const_cstr!("%.3f");
            let mut len = v.length;
            let mut acc = v.max_acc;
            let mut brk = v.max_brk;
            let mut vel = v.max_vel;
            igSliderFloat(const_cstr!("Length").as_ptr(), 
                          &mut len as *mut _, 1.0, 1000.0, format.as_ptr(), 1.0);
            if igIsItemEdited() {
                new_model.vehicles.get_mut(*i).unwrap().length = len;
                modified = Some(EditClass::VehicleLen(*i));
            }
            igSliderFloat(const_cstr!("Accel").as_ptr(), 
                          &mut acc as *mut _, 0.05, 1.5, format.as_ptr(), 1.0);
            if igIsItemEdited() {
                new_model.vehicles.get_mut(*i).unwrap().max_acc = acc;
                modified = Some(EditClass::VehicleAcc(*i));
            }
            igSliderFloat(const_cstr!("Brake").as_ptr(), 
                          &mut brk as *mut _, 0.05, 1.5, format.as_ptr(), 1.0);
            if igIsItemEdited() {
                new_model.vehicles.get_mut(*i).unwrap().max_brk = brk;
                modified = Some(EditClass::VehicleBrk(*i));
            }
            igSliderFloat(const_cstr!("Max.vel").as_ptr(), 
                          &mut vel as *mut _, 1.0, 200.0, format.as_ptr(), 1.0);
            if igIsItemEdited() {
                new_model.vehicles.get_mut(*i).unwrap().max_vel = vel;
                modified = Some(EditClass::VehicleVel(*i));
            }
        }

        igPopID();
    }

    if modified.is_some() {
        doc.viewmodel.set_model(new_model, modified);
    }

    if doc.viewmodel.model().vehicles.iter().next().is_none() {
        widgets::show_text("No vehicles defined.");
    }

    if igButton(const_cstr!("Add vehicle").as_ptr(), ImVec2 { x: 0.0, y: 0.0 }) {
        doc.viewmodel.edit_model(|m| {
            let id = m.vehicles.insert( Vehicle {
                name: String::new(),
                length: 100.0,
                max_acc: 1.0,
                max_brk: 0.5,
                max_vel: 50.0,
            });
            m.vehicles.get_mut(id).unwrap().name = format!("Vehicle {}", id);
            None
        });
    }

    }
}


pub fn edit_vehicles_window(popen :&mut bool, doc :&mut Document) {
    if !*popen { return; }
    unsafe {
    igBegin(const_cstr!("Vehicles").as_ptr(), popen as *mut bool, 0 as _);

    edit_vehicles(doc);

    igEnd();
    }
}
