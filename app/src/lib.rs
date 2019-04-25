/// Represents application state, which can be 
/// inspected (read-only) or updated by sending events.
pub struct App {
    contents :AppContents
}

pub struct AppContents {
    pub model :junc_model::Model,
    pub derived :junc_calc::DerivedModel,
    pub view :View,
}

impl App {
    /// Construct application state from model.
    pub fn new(model :junc_model::Model) -> App {
        let view = View::default_view(&model);
        let derived = junc_calc::DerivedModel::new(&model);
        App { contents: AppContents { model, derived, view } }
    }

    /// Inspect application state.
    pub fn get(&self) -> &AppContents { &self.contents }
    /// Update application state.
    pub fn integrate(&mut self, action :AppEvent) { unimplemented!() } 
}

/// 2D viewport and user interaction state.
pub struct View {
}

impl View {
    pub fn default_view(_model :&junc_model::Model) -> View {
        View {} // TODO
    }
}

pub enum AppEvent {
    Quit, // TODO ask for save?
    Model(junc_model::ModelAction),
}

