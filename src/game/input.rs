use std::str::FromStr;

use crate::game::error::{Error, GameResult};

/// The different types of io input the game can ask for.
#[derive(Debug)]
pub(super) enum Input {
    Col(usize),
    Enter,
    Yes,
    No,
    Quit,
    ToggleAi,
    Help,
    ToggleButtons,
    // AiAdvice, // TODO ask the ai for placement advice (also useful for ai debugging)
}

impl Input {
    /// Attempt to get input from stdin.
    pub(super) fn get() -> GameResult<Self> {
        let mut buf = String::new();

        std::io::stdin()
            .read_line(&mut buf)
            .expect("Failed to read stdin");
        Input::from_str(&buf)
    }
}

impl std::str::FromStr for Input {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().trim() {
            "" => Ok(Self::Enter),
            "yes" | "y" => Ok(Self::Yes),
            "no" | "n" => Ok(Self::No),
            "stop" | "exit" | "quit" | "s" | "e" | "q" => Ok(Self::Quit),
            "ai" | "toggle ai" => Ok(Self::ToggleAi),
            "help" | "h" | "?" => Ok(Self::Help),
            "button" | "buttons" => Ok(Self::ToggleButtons),
            col if col.parse::<usize>().is_ok() => match col.parse::<usize>() {
                Ok(i) => Ok(Self::Col(i)),
                Err(_) => Err(Error::InvalidInput),
            },
            _ => Err(Error::InvalidInput),
        }
    }
}
