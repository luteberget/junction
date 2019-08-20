use crate::model::*;
use crate::ui;
use const_cstr::*;
use crate::viewmodel::*;
use backend_glfw::imgui::*;

pub fn edit_vehicles(vm :&mut ViewModel) {
    unsafe {
    let mut new_model = vm.get_undoable().get().clone();
    let mut modified = false;
    for (i,v) in vm.get_undoable().get().vehicles.iter().enumerate() {
        igPushIDInt(i as _);

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
                new_model.vehicles[i].name = s;
                modified = true;
            }

            let format = const_cstr!("%.3f");
            let mut len = v.length;
            let mut acc = v.max_acc;
            let mut brk = v.max_brk;
            let mut vel = v.max_vel;
            igSliderFloat(const_cstr!("Length").as_ptr(), 
                          &mut len as *mut _, 1.0, 1000.0, format.as_ptr(), 1.0);
            if igIsItemEdited() {
                new_model.vehicles[i].length = len;
                modified = true;
            }
            igSliderFloat(const_cstr!("Accel").as_ptr(), 
                          &mut acc as *mut _, 0.05, 1.5, format.as_ptr(), 1.0);
            if igIsItemEdited() {
                new_model.vehicles[i].max_acc = acc;
                modified = true;
            }
            igSliderFloat(const_cstr!("Brake").as_ptr(), 
                          &mut brk as *mut _, 0.05, 1.5, format.as_ptr(), 1.0);
            if igIsItemEdited() {
                new_model.vehicles[i].max_brk = brk;
                modified = true;
            }
            igSliderFloat(const_cstr!("Max.vel").as_ptr(), 
                          &mut vel as *mut _, 1.0, 200.0, format.as_ptr(), 1.0);
            if igIsItemEdited() {
                new_model.vehicles[i].max_vel = vel;
                modified = true;
            }
        }

        igPopID();
    }

    if modified {
        vm.set_model(new_model);
    }

    if vm.get_undoable().get().vehicles.len() == 0 {
        ui::show_text("No vehicles defined.");
    }

    if igButton(const_cstr!("Add vehicle").as_ptr(), ImVec2 { x: 0.0, y: 0.0 }) {
        let mut new_model = vm.get_undoable().get().clone();
        let name = format!("Vehicle {}", new_model.vehicles.len() +1);
        new_model.vehicles.push_back(Vehicle {
            name: name,
            length: 100.0,
            max_acc: 1.0,
            max_brk: 0.5,
            max_vel: 50.0,
        });
        vm.set_model(new_model);
    }

    }
}


pub fn edit_vehicles_window(popen :&mut bool, vm :&mut ViewModel) {
    unsafe {
    igBegin(const_cstr!("Vehicles").as_ptr(), popen as *mut bool, 0 as _);

    edit_vehicles(vm);

    igEnd();
    }
}
