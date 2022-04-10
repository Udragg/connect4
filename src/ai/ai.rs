use rand::prelude::SliceRandom;

use crate::game::{
    board::Board,
    components::{Check4, TileType},
    error::{Error, GameResult},
};

// TODO rename
enum MoveRanking {
    WinChance(usize),
    Neutral(usize),
    OpponentWin(usize),
    ColumnFull(usize),
    NoOptions,
}

/// Ai.
pub(crate) struct Ai<const W: usize, const H: usize> {
    /// Reference to the game board.
    board: Board<W, H>,
    /// Internal board, used for testing placement positions.
    test_board: Board<W, H>,
}

impl<const W: usize, const H: usize> Ai<W, H> {
    /// Create a new ai.
    pub(crate) fn new() -> GameResult<Self> {
        Ok(Self {
            board: Board::new()?,
            test_board: Board::new()?,
        })
    }

    /// Let the AI make a move.
    ///
    /// Returns the column in which the AI decides place a piece. This column is 1 indexed.
    pub(crate) fn make_move(&mut self, board: &Board<W, H>) -> usize {
        // set the internal boards
        self.board = board.clone();
        self.test_board = board.clone();

        // check if the ai can connect 4
        if let Some(col) = self.check_make_4(TileType::Player2) {
            log::debug!("AI: Making connect 4 at column {col}");
            return col;
        }

        // check if the opponent can connect 4
        if let Some(col) = self.check_make_4(TileType::Player1) {
            log::debug!("AI: Preventing connect 4 at column {col}");
            return col;
        }

        // place in a random spot
        let mut columns = vec![0; W]
            .iter()
            .enumerate()
            .map(|(i, _)| i + 1)
            .collect::<Vec<usize>>();

        // shuffle the remaining values
        let mut rng = rand::thread_rng();
        columns.shuffle(&mut rng);

        match self.rand_place(columns) {
            MoveRanking::WinChance(col) => {
                log::debug!("AI: Attempting to win at column {col}");
                col
            }
            MoveRanking::Neutral(col) => {
                log::debug!("AI: Placing neutral at column {col}");
                col
            }
            MoveRanking::OpponentWin(col) => {
                log::debug!("AI: Oponent can win at column {col}");
                col
            }
            _ => unreachable!(),
        }
    }

    /// Check if the given player can connect 4.
    ///
    /// Returns [Some(column)](std) if yes, and [None] if no.
    /// This is used to check if the ai can connect 4, if the opposition can connect 4,
    /// or if a placement results in the opponent being able to connect 4.
    fn check_make_4(&mut self, player: TileType) -> Option<usize> {
        for col in 1..=W {
            if let Ok(()) = self.test_board.place(col, player) {
                match self.test_board.check4() {
                    Check4::None => (),
                    Check4::Draw => (),
                    Check4::Player(_) => return Some(col),
                }

                drop(self.test_board.undo_last()); // drop to discard result without warning
            }
        }
        None
    }

    /// Select a random board placement.
    ///
    /// Takes a vector of ramaining valid positions.
    ///
    /// Returns a column to place in.
    fn rand_place(&mut self, mut positions: Vec<usize>) -> MoveRanking {
        self.test_board = self.board.clone();

        // if no options remain
        let column = match positions.pop() {
            Some(col) => col,
            None => {
                log::trace!("AI: position vector empty");
                return MoveRanking::NoOptions;
            }
        };

        match self.test_board.place(column, TileType::Player2) {
            Ok(()) => {
                if let Some(col) = self.check_make_4(TileType::Player2) {
                    log::trace!("AI: chance to win at {col}");
                    MoveRanking::WinChance(col)
                } else {
                    match self.check_make_4(TileType::Player1) {
                        // placement results in opponent being able to connect 4 their next turn
                        Some(_) => match self.rand_place(positions) {
                            MoveRanking::Neutral(col) => MoveRanking::Neutral(col),
                            _ => MoveRanking::OpponentWin(column),
                        },
                        // placement doesn't result in opponent being able to connect 4 their next turn
                        None => MoveRanking::Neutral(column),
                    }
                }
            }
            // column is full
            Err(Error::ColumnFull) => match self.rand_place(positions) {
                MoveRanking::WinChance(col) => MoveRanking::WinChance(col),
                MoveRanking::Neutral(col) => MoveRanking::Neutral(col),
                MoveRanking::OpponentWin(col) => MoveRanking::OpponentWin(col),
                _ => MoveRanking::ColumnFull(column),
            },
            //
            _ => unimplemented!(),
        }

        // match self.test_board.place(
        //     *remaining.last().expect("vector should have at least 1 element"),
        //     TileType::Player2,
        // ) {
        //     Ok(()) => *remaining.last().expect("vector should have at least 1 element"),
        //     Err(Error::ColumnFull) => {
        //         remaining.pop();
        //         self.rand_place(remaining)
        //     }
        //     Err(_) => unimplemented!(),
        // }
    }
}
