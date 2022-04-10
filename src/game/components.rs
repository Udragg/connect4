/// The type of tiles that can be on the board.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TileType {
    Player1,
    Player2,
    Empty,
}

/// If there has been a winner or a draw.
#[derive(Debug)]
pub(crate) enum Check4 {
    Player([(usize, usize); 4]),
    Draw,
    None,
}

#[derive(Debug)]
pub(super) enum ActivePlayer {
    Player1,
    Player2,
    Ai,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum AiState {
    Enabled,
    Disabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ButtonState {
    Enabled,
    Disabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DisplayState {
    Enabled,
    Disabled,
}

/// Struct containing the players.
#[derive(Debug)]
pub(super) struct Players {
    pub player1: Player,
    pub player2: Player,
    pub active: ActivePlayer,
}

impl Players {
    /// Create a new players.
    pub(super) fn new(player1: Player, player2: Player) -> Self {
        Self {
            player1,
            player2,
            active: ActivePlayer::Player1,
        }
    }

    /// Get the current active player.
    pub(super) fn active(&self) -> Player {
        match self.active {
            ActivePlayer::Player1 => self.player1.clone(),
            ActivePlayer::Player2 => self.player2.clone(),
            ActivePlayer::Ai => Player::ai_placeholder(),
        }
        // if self.active {
        //     self.player1.clone()
        // } else {
        //     self.player2.clone()
        // }
    }

    /// Add a point to the active player.
    pub(super) fn scored(&mut self) {
        match self.active {
            ActivePlayer::Player1 => self.player1.score += 1,
            ActivePlayer::Player2 => self.player2.score += 1,
            ActivePlayer::Ai => (), // TODO track ai score
        }
    }

    pub(super) fn swap(&mut self) {
        self.active = match self.active {
            ActivePlayer::Player1 => ActivePlayer::Player2,
            ActivePlayer::Player2 => ActivePlayer::Player1,
            ActivePlayer::Ai => unimplemented!(),
        }
    }

    pub(super) fn swap_ai(&mut self) {
        self.active = match self.active {
            ActivePlayer::Player1 => ActivePlayer::Ai,
            ActivePlayer::Ai => ActivePlayer::Player1,
            ActivePlayer::Player2 => unimplemented!(),
        }
    }

    pub(super) fn reset_scores(&mut self) {
        self.player1.score = 0;
        self.player2.score = 0;
    }

    pub(super) fn set_active(&mut self, active: ActivePlayer) {
        self.active = active;
    }
}

/// A single player
#[derive(Debug, Clone)]
pub(super) struct Player {
    pub name: String,
    pub score: usize,
    pub color: c4_display::LedColor,
    pub tile: TileType,
}

impl Player {
    pub(crate) fn ai_placeholder() -> Self {
        Self {
            name: "AI".to_string(),
            score: 0,
            color: c4_display::LedColor::Yellow,
            tile: TileType::Player2,
        }
    }
}

impl Default for TileType {
    fn default() -> Self {
        Self::Empty
    }
}

impl std::fmt::Display for Player {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
