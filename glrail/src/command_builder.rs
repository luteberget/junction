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

pub struct ArgumentListBuilder {
}

pub struct MiniMode {
}


