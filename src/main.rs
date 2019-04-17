/// Entry point for Junction
/// Creates an empty App from the app crate,
/// and hands it to the GUI function from the ui crate.

pub fn main() {
    // TODO command line options

    let model   = junc_model::Model::empty();         // Model
    let calc    = junc_calc::Calc::from_model(model); // Extended (non-editable) model
    let app     = junc_app::App::new(calc);           // View model
    junc_gui::run(app);                               // View
} 

