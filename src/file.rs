use crate::document::model::Model;
use std::fs::File;
use log::*;

pub fn load(filename :&str) -> Result<Model, std::io::Error> {
    let m = serde_cbor::from_reader(File::open(&filename)?)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    Ok(m)
}

pub fn save(filename :&str, m :Model) -> Result<(),std::io::Error> {
    info!("Will save file to file name {:?}", filename);
    serde_cbor::to_writer(&File::create(filename)?, &m)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    Ok(())
}

pub fn save_interactive(m :Model) -> Result<Option<String>,std::io::Error> {
    if let Some(filename) = tinyfiledialogs::save_file_dialog("Save model to file", "") {
        save(&filename, m).map(|_| Some(filename))
    } else {
        Ok(None) // user cancelled, this is not an error
    }
}

pub fn load_interactive() -> Result<Option<(Model,String)>, std::io::Error> {
    if let Some(filename) = tinyfiledialogs::open_file_dialog("Open model from file", "", None) {
        info!("Loading file from {:?}", filename);
        let m = load(&filename)?;
        Ok(Some((m,filename)))
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

    pub fn set_saved_file(&mut self, filename :String) {
        self.unsaved = false;
        self.filename = Some(filename);
        self.update_window_title();
    }

    pub fn set_saved(&mut self) {
        self.unsaved = false;
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
