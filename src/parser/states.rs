pub enum Action {
    Ignore,
    Append,
    Carry,
}

pub trait State {
    fn can_terminate(&self) -> bool { false }

    fn accept(&self, grapheme: &str) -> (Action, Box<dyn State>);
}

pub struct Start;
pub struct InString;
pub struct InDelimitedIdentifier;
pub struct MightStartInlineComment;
pub struct InInlineComment;
pub struct MightStartBlockComment;
pub struct InBlockComment;
pub struct MightEndBlockComment;


impl State for Start { // 1
    fn can_terminate(&self) -> bool {
        true
    }
    
    fn accept(&self, s: &str) -> (Action, Box<dyn State>) {
        match s {
            "'"  => (Action::Append, Box::new(InString)),
            "\"" => (Action::Append, Box::new(InDelimitedIdentifier)),
            "-"  => (Action::Carry,  Box::new(MightStartInlineComment)),
            "/"  => (Action::Carry,  Box::new(MightStartBlockComment)),
            _    => (Action::Append, Box::new(Start)),

        }
    }
}

impl State for InString { // 2
    fn accept(&self, s: &str) -> (Action, Box<dyn State>) {
        match s {
            "'" => (Action::Append, Box::new(Start)),
            _   => (Action::Append, Box::new(InString)),

        }
    }
}

impl State for InDelimitedIdentifier { // 3
    fn accept(&self, s: &str) -> (Action, Box<dyn State>) {
        match s {
            "\"" => (Action::Append, Box::new(Start)),
            _    => (Action::Append, Box::new(InDelimitedIdentifier)),

        }
    }
}

impl State for MightStartInlineComment { // 4
    fn can_terminate(&self) -> bool {
        true
    }

    fn accept(&self, s: &str) -> (Action, Box<dyn State>) {
        match s {
            "'"  => (Action::Append, Box::new(InString)),
            "\"" => (Action::Append, Box::new(InDelimitedIdentifier)),
            "--" => (Action::Ignore, Box::new(InInlineComment)),
            "/"  => (Action::Carry,  Box::new(MightStartBlockComment)),
            _    => (Action::Append, Box::new(Start)),
        }
    }
}

impl State for InInlineComment { // 5
    fn accept(&self, s: &str) -> (Action, Box<dyn State>) {
        match s {
            "\n" => (Action::Append, Box::new(Start)),
            _    => (Action::Ignore, Box::new(InInlineComment)),
        }
    }
}

impl State for MightStartBlockComment { // 6
    fn can_terminate(&self) -> bool {
        true
    }

    fn accept(&self, s: &str) -> (Action, Box<dyn State>) {
        match s {
            "'"  => (Action::Append, Box::new(InString)),
            "\"" => (Action::Append, Box::new(InDelimitedIdentifier)),
            "-"  => (Action::Ignore, Box::new(MightStartInlineComment)),
            "/*" => (Action::Carry,  Box::new(InBlockComment)),
            _    => (Action::Append, Box::new(Start)),
        }
    }
}

impl State for InBlockComment { // 7
    fn accept(&self, s: &str) -> (Action, Box<dyn State>) {
        match s {
            "*" => (Action::Carry,  Box::new(MightEndBlockComment)),
            _   => (Action::Ignore, Box::new(InBlockComment)),
        }
    }
}

impl State for MightEndBlockComment { // 8
    fn accept(&self, s: &str) -> (Action, Box<dyn State>) {
        match s {
            "*/" => (Action::Ignore, Box::new(Start)),
            _    => (Action::Ignore, Box::new(InBlockComment)),
        }
    }
}
