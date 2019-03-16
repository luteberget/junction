use serde_json;
use crate::text::*;
use crate::sys::*; 
use const_cstr::*;

pub struct OpenObject {
    pub newkey :String,
    pub open_subobjects :Vec<(String, Box<OpenObject>)>,
}

type UserData = serde_json::Map<String, serde_json::Value>;

pub fn json_editor(types: &[*const i8; 6], data :&mut UserData, open :&mut OpenObject) {
    let v2_0 = ImVec2 { x: 0.0, y : 0.0 };
    unsafe {
        let mut del = None;
        for (i,(k,v)) in data.iter_mut().enumerate() {
            igPushIDInt(i as _);
            show_text(k);

            if igButton(const_cstr!("\u{f056}").as_ptr(), v2_0) {
                del = Some(k.clone());
            }
            igSameLine(0.0, -1.0);
            
            igPushItemWidth(3.0*16.0);

            let l_null = const_cstr!("null");
            let l_bool = const_cstr!("bool");
            let l_number = const_cstr!("num");
            let l_text = const_cstr!("text");
            let l_array = const_cstr!("arr");
            let l_object = const_cstr!("obj");

            let curr_type_str = match v {
                             serde_json::Value::Null => l_null,
                             serde_json::Value::Bool(_) => l_bool,
                             serde_json::Value::Number(_) => l_number,
                             serde_json::Value::String(_) => l_text,
                             serde_json::Value::Object(_) => l_object,
                             serde_json::Value::Array(_) => l_array,
                             _ => l_text,
                         };

            if igBeginCombo(const_cstr!("##type").as_ptr(), curr_type_str.as_ptr(),
                         ImGuiComboFlags__ImGuiComboFlags_NoArrowButton as _) {

                if igSelectable(l_null.as_ptr(), l_null == curr_type_str, 0, v2_0) 
                    && l_null != curr_type_str {
                        *v = serde_json::Value::Null;
                }
                if igSelectable(l_bool.as_ptr(), l_bool == curr_type_str, 0, v2_0) 
                    && l_bool != curr_type_str {
                        *v = serde_json::Value::Bool(Default::default());
                }
                if igSelectable(l_number.as_ptr(), l_number == curr_type_str, 0, v2_0) 
                    && l_number != curr_type_str {
                        *v = serde_json::Value::Number(serde_json::Number::from_f64(0.0).unwrap());
                }
                if igSelectable(l_text.as_ptr(), l_text == curr_type_str, 0, v2_0) 
                    && l_text != curr_type_str {
                        *v = serde_json::Value::String(Default::default());
                }
                if igSelectable(l_array.as_ptr(), l_array == curr_type_str, 0, v2_0) 
                    && l_array != curr_type_str {
                        *v = serde_json::Value::Array(Default::default());
                }
                if igSelectable(l_object.as_ptr(), l_object == curr_type_str, 0, v2_0) 
                    && l_object != curr_type_str {
                        *v = serde_json::Value::Object(Default::default());
                }
                igEndCombo();
            }
            igPopItemWidth();

            igPushItemWidth(-1.0);

            match v {
                serde_json::Value::Null => {},
                serde_json::Value::Bool(ref mut b) => {
                    let l_true = const_cstr!("true");
                    let l_false = const_cstr!("false");
                    igSameLine(0.0, -1.0);
                    if igBeginCombo(const_cstr!("##bool").as_ptr(), 
                                    (if *b { l_true } else { l_false }).as_ptr(),0) {

                        if igSelectable(l_false.as_ptr(), !*b, 0, v2_0) && *b {
                            *b = false;
                        }
                        if igSelectable(l_true.as_ptr(), *b, 0, v2_0) && !*b {
                            *b = true;
                        }
                        igEndCombo();
                    }
                },
                serde_json::Value::Number(ref mut n) => {
                    let mut num : f32 = n.as_f64().unwrap() as _;
                    igSameLine(0.0, -1.0);
                    igInputFloat(const_cstr!("##num").as_ptr(), 
                                 &mut num as *mut _, 0.0, 1.0, 
                                 const_cstr!("%g").as_ptr(), 0);
                    if igIsItemDeactivatedAfterEdit() {
                        *n = serde_json::Number::from_f64(num as _).unwrap();
                    }
                },
                serde_json::Value::String(ref mut s) => {
                    igSameLine(0.0, -1.0);
                    input_text_string(
                        const_cstr!("##text").as_cstr(), 
                        Some(const_cstr!("empty").as_cstr()), 
                        s, 0);
                },
                serde_json::Value::Array(ref mut a) => {
                    igSameLine(0.0, -1.0);
                    if igTreeNodeStr(const_cstr!("Array").as_ptr()) {
                        igText(const_cstr!("...").as_ptr());
                        igTreePop();
                    }
                },
                serde_json::Value::Object(ref mut o) => {
                    igSameLine(0.0, -1.0);
                    if igTreeNodeStr(const_cstr!("Object").as_ptr()) {

                        //json_editor
                        json_editor(&types, o, open);

                        igTreePop();
                    }
                },
                _ => unimplemented!(),
            }

            igPopItemWidth();
            //println!("{:?}: {:?}", k,v);
            igPopID();
        }

        if let Some(k) = del {
            data.remove(&k);
        }

        if igButton(const_cstr!("\u{f055}").as_ptr(), ImVec2 { x: 0.0, y: 0.0 })  {
            use std::mem;
            let s = &mut open.newkey;
            if s.len() > 0 {
                data.insert(
                    mem::replace(s, String::new()),
                    serde_json::Value::Null);
            }
        }

       igSameLine(0.0, -1.0);
       input_text_string( const_cstr!("##newkey").as_cstr(), 
                   Some(const_cstr!("New key").as_cstr()), &mut open.newkey, 0);
    }
}


