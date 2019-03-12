use smallvec::SmallVec;
use crate::app::*;
pub struct CommandBuilder {
    menu_stack :SmallVec<[CommandScreen; 4]>,
}

impl CommandBuilder {
    pub fn new_menu(menu :Menu) -> Self {
        let mut menu_stack = SmallVec::new();
        menu_stack.push(CommandScreen::Menu(menu));
        CommandBuilder { menu_stack }
    }
    pub fn new_screen(screen :CommandScreen) -> Self {
        let mut menu_stack = SmallVec::new();
        menu_stack.push(screen);
        CommandBuilder { menu_stack }
    }

    pub fn execute(mut self, app: &mut App) {
        if let CommandScreen::ArgumentList(mut alb) = self.menu_stack.pop().unwrap() {
            alb.execute(app);
        }
    }

    pub fn current_screen(&mut self) -> &mut CommandScreen {
        let l = self.menu_stack.len();
        &mut self.menu_stack[l-1]
    }

    pub fn push_screen(&mut self, s :CommandScreen) {
        self.menu_stack.push(s);
    }
}

pub enum CommandScreen {
    Menu(Menu),
    ArgumentList(ArgumentListBuilder),
    MiniMode(MiniMode),
    Action(&'static fn(app :&mut App)),
    Close,
}

#[derive(Clone)]
pub struct Menu {
    pub choices :Vec<(char, String, 
                      fn(app :&mut App) -> Option<CommandScreen>)>,
}

pub enum Arg {
    Id(Option<EntityId>),
    Float(f32),
}

pub enum ArgStatus {
    Done, NotDone,
}

pub struct ArgumentListBuilder {
    pub arguments : Vec<(String, ArgStatus, Arg)>,
    pub function: Option<fn(app :&mut App, alb :&ArgumentListBuilder)>,
}

impl ArgumentListBuilder {
    pub fn new() -> ArgumentListBuilder {
        ArgumentListBuilder {
            arguments: vec![],
            function: None,
        }
    }

    pub fn get_float(&self, name :&str) -> Option<&f32> {
        for (n,s,a) in &self.arguments {
            if n == name {
                return match a {
                    Arg::Float(x) => Some(x),
                    _ => None,
                }
            } 
        }
        None
    }

    pub fn get_id(&self, name :&str) -> Option<&EntityId> {
        for (n,s,a) in &self.arguments {
            if n == name {
                return match a {
                    Arg::Id(Some(x)) => Some(x),
                    _ => None,
                }
            }
        }
        None
    }

    pub fn set_action(&mut self, f :fn(app :&mut App, alb :&ArgumentListBuilder)) {
        self.function = Some(f);
    }

    pub fn execute(&mut self, app :&mut App) {
        // TODO: should this function return a new screen?
        if let Some(f) = self.function {
            f(app, self);
        }
    }

    pub fn add_id_value(&mut self, name : impl Into<String>, id :EntityId) {
        self.arguments.push((name.into(), ArgStatus::Done, Arg::Id(Some(id))));
    }

    pub fn add_id(&mut self, name : impl Into<String>) {
        self.arguments.push((name.into(), ArgStatus::NotDone, Arg::Id(None)));
    }

    pub fn add_float_default(&mut self, name : impl Into<String>, val :f32) {
        self.arguments.push((name.into(), ArgStatus::NotDone, Arg::Float(val)));
    }
}

pub struct MiniMode {
}


