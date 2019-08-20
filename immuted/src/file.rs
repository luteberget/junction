use crate::model::Model;
use std::fs::File;
use log::*;

pub fn save(fileinfo :&mut FileInfo, filename :String, m :Model) -> Result<(),std::io::Error> {
    info!("Will save file to file name {:?}", filename);
    serde_cbor::to_writer(&File::create(&filename)?, &m)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    fileinfo.set_saved(filename);
    Ok(())
}

pub fn save_interactive(fileinfo :&mut FileInfo, m :Model) -> Result<(),std::io::Error> {
    if let Some(filename) = fileinfo.filename.clone() {
        save(fileinfo, filename, m)
    } else {
        save_as_interactive(fileinfo,m)
    }
}

pub fn save_as_interactive(fileinfo :&mut FileInfo, m :Model) -> Result<(),std::io::Error> {
    if let Some(filename) = tinyfiledialogs::save_file_dialog("Save model to file", "") {
        save(fileinfo, filename, m)
    } else {
        Ok(()) // user cancelled, this is not an error
    }
}

pub fn load_doc(fileinfo :&mut FileInfo) -> Result<Option<Model>, std::io::Error> {
    if let Some(filename) = tinyfiledialogs::open_file_dialog("Open model from file", "", None) {
        info!("Loading file from {:?}", filename);
        let m = serde_cbor::from_reader(File::open(&filename)?)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        fileinfo.set_saved(filename);
        Ok(Some(m))
    } else {
        Ok(None)
    }
}

#[derive(Debug)]
#[derive(Clone)]
pub struct FileInfo {
    pub filename :Option<String>,
    pub unsaved :bool,
}

impl FileInfo {
    pub fn empty() -> Self {
        FileInfo {
            filename :None,
            unsaved :false,
        }
    }

    pub fn set_saved(&mut self, filename :String) {
        self.unsaved = false;
        self.filename = Some(filename);
        self.update_window_title();
    }

    pub fn set_unsaved(&mut self) {
        if !self.unsaved {
            self.unsaved = true;
            self.update_window_title();
        }
    }

    pub fn update_window_title(&self) {
        backend_glfw::set_window_title(&self.window_title());
    }

    pub fn window_title(&self) -> String {
        format!("{}{} - Junction", if self.unsaved {"*"}  else { "" },
                                   self.filename.as_ref().map(|x| x.as_str()).unwrap_or("Untitled"))
    }
}
