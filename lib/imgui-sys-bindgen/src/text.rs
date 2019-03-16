use crate::sys::*;
use std::ffi::CStr;

pub fn input_text_string(
    label: &CStr,
    hint: Option<&CStr>,
    buffer: &mut String,
    flags: ImGuiInputTextFlags) {
    buffer.push('\0');
    input_text(label,hint, unsafe { buffer.as_mut_vec() },flags);
    buffer.pop();
}


pub fn input_text(
    label: &CStr,
    hint: Option<&CStr>,
    buffer: &mut Vec<u8>,
    mut flags: ImGuiInputTextFlags) {

   unsafe extern "C" fn resize_func(data: *mut ImGuiInputTextCallbackData) -> std::os::raw::c_int  {
       //println!("BufTextLen {:?}", (*data).BufTextLen);
       let vecptr = ((*data).UserData as *mut Vec<u8>);
       (*vecptr).resize((*data).BufTextLen as usize + 1, '\0' as u8);
       (*vecptr)[(*data).BufTextLen as usize ] = '\0' as u8;
       (*data).Buf = (*vecptr).as_mut_ptr() as _;
       0
   }

   match hint {
       Some(hint) => {
           unsafe {
           igInputTextWithHint(
               label.as_ptr(),
                //const_cstr!("").as_ptr(),
                //const_cstr!("New key").as_ptr(),
               hint.as_ptr(),
                buffer.as_mut_ptr() as _,
                buffer.capacity()+1,
                flags | (ImGuiInputTextFlags__ImGuiInputTextFlags_CallbackResize as ImGuiInputTextFlags) ,
                Some(resize_func),
                buffer as *mut _ as _);
           }
       },
       None => {
           // TODO igInputText
           unimplemented!()
       }
   }

}

pub fn show_text(s :&str) {
    unsafe {
    igTextSlice(s.as_ptr() as _ , s.as_ptr().offset(s.len() as _ ) as _ );
    }
}

