pub use rolling::input::staticinfrastructure::Route;

pub struct Interlocking {
    // TODO indexing on boundary/signal
    pub routes :Vec<Route>,
}
