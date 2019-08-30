use crate::app::*;
use crate::document::dispatch::*;
use crate::gui::widgets;

pub fn diagram_view(app :&App, dv :&DispatchView) {
    widgets::show_text(&format!("Diapsathc {:?}", dv));
}
