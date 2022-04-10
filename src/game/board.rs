use super::{
    components::{Check4, TileType},
    error::{Error, GameResult},
};

#[derive(Debug, Clone)]
pub(crate) struct Board<const W: usize, const H: usize> {
    board: [[TileType; W]; H],
    last_move: Option<(usize, usize, TileType)>, // x, y, previous tile // TODO move history vec
    selected: usize,
    active_type: TileType,
}

impl<const W: usize, const H: usize> Board<W, H> {
    /// Create a new Board.
    pub(crate) fn new() -> GameResult<Self> {
        if W < 4 || H < 5 {
            return Err(Error::InvalidDim);
        }
        let mut board = [[TileType::default(); W]; H];
        board[0][0] = TileType::Player1;
        Ok(Self {
            board,
            last_move: None,
            selected: 0,
            active_type: TileType::Empty,
        })
    }

    /// Place a tile in the given column. The column is zero indexed.
    ///
    /// # Errors
    ///
    /// Returns `Error::InvalidColumn` if the column is outside the board dimensions.
    ///
    /// Returns `Error::InvalidType` if the tile is of type `TileType::Empty`.
    ///
    /// Returns `Error::ColumnFull` if it failed to place the tile anywhere in the column.
    pub(crate) fn place(&mut self, col: usize, tile: TileType) -> GameResult<()> {
        if col < 1 || col > W {
            return Err(Error::InvalidColumn);
        } else if tile == TileType::Empty {
            return Err(Error::InvalidType);
        }
        for y in (1..H).rev() {
            if self.board[y][col - 1] == TileType::Empty {
                self.last_move = Some((col - 1, y, self.board[y][col - 1]));
                self.board[y][col - 1] = tile;
                return Ok(());
            }
        }
        Err(Error::ColumnFull)
    }

    /// Check if any player has 4 connected tiles.
    ///
    /// Returns if a player won or if there is a draw.
    pub(crate) fn check4(&self) -> Check4 {
        let mut draw = true;
        for y in 1..H {
            for x in 0..W {
                let tile = self.board[y][x];
                if tile == TileType::Empty {
                    draw = false;
                } else {
                    // check right
                    if x < W - 3
                        && self.board[y][x + 1] == tile
                        && self.board[y][x + 2] == tile
                        && self.board[y][x + 3] == tile
                    {
                        return Check4::Player([(y, x), (y, x + 1), (y, x + 2), (y, x + 3)]);
                    }

                    if y > 2 {
                        // check up
                        if self.board[y - 1][x] == tile
                            && self.board[y - 2][x] == tile
                            && self.board[y - 3][x] == tile
                        {
                            return Check4::Player([(y, x), (y - 1, x), (y - 2, x), (y - 3, x)]);
                        }

                        // check up & right
                        if x < W - 3
                            && self.board[y - 1][x + 1] == tile
                            && self.board[y - 2][x + 2] == tile
                            && self.board[y - 3][x + 3] == tile
                        {
                            return Check4::Player([
                                (y, x),
                                (y - 1, x + 1),
                                (y - 2, x + 2),
                                (y - 3, x + 3),
                            ]);
                        }

                        // check up & left
                        if x > 2
                            && self.board[y - 1][x - 1] == tile
                            && self.board[y - 2][x - 2] == tile
                            && self.board[y - 3][x - 3] == tile
                        {
                            return Check4::Player([
                                (y, x),
                                (y - 1, x - 1),
                                (y - 2, x - 2),
                                (y - 3, x - 3),
                            ]);
                        }
                    }
                }
            }
        }
        if draw {
            return Check4::Draw;
        }
        Check4::None
    }

    /// Reset the board to its original state.
    pub(super) fn reset(&mut self) {
        self.board = [[TileType::default(); W]; H]
    }

    /// Get the type of tile at position (x, y).
    pub(crate) fn get(&self, x: usize, y: usize) -> GameResult<TileType> {
        if x >= W || y >= H {
            return Err(Error::InvalidDim);
        }

        Ok(self.board[y][x])
    }

    pub(crate) fn selected_left(&mut self) {
        log::trace!("moving selected left");
        self.unset_active();
        self.selected += W;
        self.selected -= 1;
        self.selected %= W;
        self.set_active(self.active_type);
    }

    pub(crate) fn selected_right(&mut self) {
        log::trace!("moving selected right");
        self.unset_active();
        self.selected += 1;
        self.selected %= W;
        self.set_active(self.active_type);
    }

    pub(crate) fn place_selected(&mut self) -> GameResult<()> {
        log::trace!("placing at selected");
        self.place(self.selected + 1, self.active_type)
    }

    pub(crate) fn set_active(&mut self, active: TileType) {
        self.active_type = active;
        self.board[0][self.selected] = active;
    }

    pub(crate) fn unset_active(&mut self) {
        self.board[0][self.selected] = TileType::Empty;
    }

    pub(crate) fn undo_last(&mut self) -> GameResult<()> {
        match self.last_move {
            Some((x, y, t)) => {
                self.board[y][x] = t;
                self.last_move = None;
                Ok(())
            }
            None => Err(Error::NoUndos),
        }
    }
}

impl<const W: usize, const H: usize> std::fmt::Display for Board<W, H> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#")?;
        for x in 1..=W {
            if x < 10 {
                write!(f, "-{x}-")?;
            } else if x < 100 {
                write!(f, "{x}-")?;
            } else {
                write!(f, "{x}")?;
            }
        }
        writeln!(f, "#")?;
        for y in 1..H {
            write!(f, "|")?;
            for x in 0..W {
                match self.board[y][x] {
                    TileType::Empty => write!(f, " . ")?,
                    TileType::Player1 => write!(f, " x ")?,
                    TileType::Player2 => write!(f, " o ")?,
                }
            }
            writeln!(f, "|")?;
        }
        write!(f, "#")?;
        for _ in 1..=W {
            write!(f, "---")?;
        }
        writeln!(f, "#")
    }
}
