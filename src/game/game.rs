use std::time::Duration;

use c4_display::{
    spin_wait, Animation, AnimationFrame, BlinkInfo, DisplayInterface, LedColor, LedState,
    PinConfig, Rotation, Running, SyncType,
};

use crate::{
    ai::Ai,
    game::{
        board::Board,
        button::ButtonsAsync,
        components::{
            ActivePlayer, AiState, ButtonState, Check4, DisplayState, Player, Players, TileType,
        },
        error::{Error, GameResult},
        input::{InputHandler, InputValue},
    },
};

/// Game manager struct.
pub struct Game<'g, const W: usize, const H: usize> {
    board: Board<W, H>,
    ai: Ai<W, H>,
    players: Players,
    display: Option<DisplayInterface<'g, Running, W, H>>,
    input_handler: InputHandler,
    ai_state: AiState,
    button_state: ButtonState,
    display_state: DisplayState,
}

impl<'g, const W: usize, const H: usize> Game<'g, W, H> {
    /// Create a new game manager instance.
    ///
    /// # Error
    ///
    /// Returns an error if the height or width constants are smaller than 4.
    pub fn new() -> GameResult<Self> {
        let board = Board::<W, H>::new()?;
        Ok(Self {
            ai: Ai::new()?,
            board,
            // TODO better player management system
            players: Players::new(
                Player {
                    name: String::from("a"),
                    score: 0,
                    color: LedColor::Red,
                    tile: TileType::Player1,
                },
                Player {
                    name: String::from("b"),
                    score: 0,
                    color: LedColor::Yellow,
                    tile: TileType::Player2,
                },
            ),
            display: None,
            ai_state: AiState::Disabled,
            input_handler: InputHandler::new(),
            button_state: ButtonState::Disabled,
            display_state: DisplayState::Disabled,
        })
    }

    /// Enable led matrix display
    pub fn enable_display(&mut self) {
        self.display = Some(DisplayInterface::new("connect4").start(
            60.0,
            PinConfig {
                sr_serin: 17,
                sr_srclk: 22,
                sr_rclk: 23,
                sr_srclr: 24,
                sr_oe: 27,
                dec_a0: 25,
                dec_a1: 11,
                dec_a2: 5,
                dec_le: 6,
                dec_e1: 10,
            },
        ));
        self.display_state = DisplayState::Enabled;
    }

    /// Disable led matrix display
    pub fn disable_display(&mut self) {
        let disp = self.display.take();

        if disp.is_some() {
            disp.unwrap().stop();
        }
    }

    /// Enable push buttons on the matrix board
    pub fn enable_buttons(&mut self) -> GameResult<()> {
        if self.button_state == ButtonState::Disabled {
            self.input_handler.start_buttons(2, 4, 3, 15, 14)?;
            // self.buttons = Some(ButtonsAsync::new(2, 4, 3, 15, 14)?);
            self.button_state = ButtonState::Enabled;
        }
        Ok(())
    }

    /// Disable push buttons on the matrix board
    pub fn disable_buttons(&mut self) {
        if self.button_state == ButtonState::Enabled {
            self.input_handler.stop_buttons();
            // self.buttons.take();
            self.button_state = ButtonState::Disabled;
        }
    }

    /// Enable the ai opponent.
    pub fn enable_ai(&mut self) {
        self.ai_state = AiState::Enabled;
    }

    /// Disable the ai opponent.
    pub fn disable_ai(&mut self) {
        self.ai_state = AiState::Disabled;
    }

    // ! win_pos should be removed once the animation system is implemented, and be done instead via a disp.add_animation() method (or similar) in start_round()
    // fn update_disp(&mut self, win_pos: Option<[(usize, usize); 4]>) {
    fn update_disp(&mut self) {
        if self.display_state == DisplayState::Enabled {
            let mut temp = vec![vec![LedState::default(); W]; H];

            for y in 0..H {
                for x in 0..W {
                    match self.board.get(x, y) {
                        Ok(TileType::Player1) => temp[y][x].color = self.players.player1.color,
                        Ok(TileType::Player2) => temp[y][x].color = self.players.player2.color,
                        Ok(TileType::Empty) => temp[y][x] = LedState::default(),
                        Err(_) => (),
                    }
                }
            }

            self.display
                .as_mut()
                .unwrap()
                .sync(SyncType::All(temp))
                .unwrap();

            // self.display
            //     .as_mut()
            //     .unwrap()
            //     .sync(SyncType::Rotate(Rotation::Clockwise))
            //     .unwrap();
        }
    }

    /// Start new round
    fn start_round(&mut self) {
        if self.display_state == DisplayState::Enabled {
            self.display.as_mut().unwrap().clear_animations();
        }

        self.board.set_active(self.players.active().tile);
        self.update_disp();

        'main: loop {
            print!("{}", self.board);
            println!("{}'s turn.", self.players.active().name);

            match self.players.active {
                ActivePlayer::Player1 | ActivePlayer::Player2 => loop {
                    match self.input_handler.get() {
                        Ok(val) => match val {
                            InputValue::BtnLeft => {
                                self.board.selected_left();
                                self.update_disp();
                            }
                            InputValue::BtnRight => {
                                self.board.selected_right();
                                self.update_disp();
                            }
                            InputValue::BtnCenter => match self.board.place_selected() {
                                Ok(()) => {
                                    self.drop_ani(self.board.selected());
                                    break;
                                }
                                Err(_) => (),
                            },
                            InputValue::Col(col) => {
                                match self.board.place(col, self.players.active().tile) {
                                    Ok(()) => {
                                        self.drop_ani(col - 1);
                                        break;
                                    }
                                    Err(Error::ColumnFull) => {
                                        println!("Column {col} is already full!")
                                    }
                                    Err(Error::InvalidColumn) => {
                                        println!("Column {col} does not exist!")
                                    }
                                    Err(Error::InvalidType) => {
                                        println!(
                                            "Cant place tile of type {:?}",
                                            self.players.active().tile
                                        )
                                    }
                                    _ => unimplemented!(),
                                }
                            }
                            InputValue::Help => {
                                print!("Place a piece in a column by typing a number between 1 and {W}");
                                println!(" (the column numbers are visible above the columns)");
                                println!("Type quit to stop the round");
                            }
                            InputValue::Quit => break 'main,
                            _ => (),
                        },
                        Err(err) => match err {
                            Error::InvalidInput(str) => println!(
                                "Invalid input: {str}\nInput must be a number between 1 and {W}"
                            ),
                            _ => panic!("expected Error::InvalidInput, found {err}"),
                        },
                    }
                },
                ActivePlayer::Ai => {
                    print!("{}", self.board);
                    let ai_move = self.ai.make_move(&self.board);
                    self.board
                        .place(ai_move, TileType::Player2)
                        .expect("Ai move invalid");
                    self.drop_ani(ai_move - 1);
                    println!("AI placed in column {ai_move}");
                }
            }

            match self.board.check4() {
                Check4::Player(pos) => {
                    self.update_disp();
                    if self.display_state == DisplayState::Enabled {
                        let mut leds = Vec::with_capacity(4);
                        for led in pos {
                            let state = LedState {
                                color: self.players.active().color,
                                blink: Some(BlinkInfo {
                                    dur: Duration::from_millis(250),
                                    int: Duration::from_millis(500),
                                }),
                            };
                            leds.push((led.1, led.0, state));
                        }
                        let frame = AnimationFrame::new(Duration::from_millis(500), leds, true);
                        let animation = Animation::new(true, vec![frame], 0, false);

                        self.display
                            .as_mut()
                            .unwrap()
                            .add_animation(animation)
                            .unwrap();
                    }

                    print!("{}", self.board);
                    self.players.scored();
                    println!("{} wins", self.players.active().name);
                    break;
                }
                Check4::Draw => {
                    print!("{}", self.board);
                    println!("Draw");
                    break;
                }
                Check4::None => {
                    match self.ai_state {
                        AiState::Enabled => self.players.swap_ai(),
                        AiState::Disabled => self.players.swap(),
                    }
                    self.board.set_active(self.players.active().tile);
                }
            }

            self.update_disp();
        }

        match self.ai_state {
            AiState::Enabled => self.players.set_active(ActivePlayer::Player1),
            AiState::Disabled => {
                self.players.swap();
                self.board.set_active(self.players.active().tile);
                println!(
                    "\n{}'s score: {}\t{}'s score: {}",
                    self.players.player1.name,
                    self.players.player1.score,
                    self.players.player2.name,
                    self.players.player2.score,
                );
            }
        }

        self.board.reset();

        if self.display_state == DisplayState::Enabled {
            loop {
                match self.input_handler.get() {
                    Ok(InputValue::Enter) | Ok(InputValue::BtnCenter) | Ok(InputValue::Quit) => {
                        break
                    }
                    _ => (),
                }
            }

            self.display.as_mut().unwrap().clear_animations();
            self.update_disp();

            self.display
                .as_mut()
                .unwrap()
                .add_animation(Animation::from_file("./animations/circle.mtxani").unwrap())
                .unwrap();
        }
    }

    /// x is 0 indexed
    fn drop_ani(&mut self, x: usize) {
        const FRAMEDUR: Duration = Duration::from_millis(100);

        let y_end = match self.board.first_empty(x) {
            Ok(y) => y,
            Err(_) => return,
        };
        let mut frames = Vec::new();

        // start at 1 to avoid statusbar
        for y in 1..y_end + 1 {
            let frame = AnimationFrame::new(
                FRAMEDUR,
                vec![(x, y, LedState::with_color(self.players.active().color))],
                true,
            );
            frames.push(frame);
        }
        let sleep_time = FRAMEDUR * frames.len() as u32;
        let ani = Animation::new(false, frames, 0, false);
        if let Some(disp) = &mut self.display {
            disp.add_animation(ani).unwrap();
            std::thread::sleep(sleep_time);
        }
    }

    /// Start the game.
    pub fn start(&mut self) {
        if self.display_state == DisplayState::Enabled {
            self.display
                .as_mut()
                .unwrap()
                .add_animation(Animation::from_file("./animations/circle.mtxani").unwrap())
                .unwrap();
        }

        loop {
            println!("Start new round? [Y/n]\t(type \"help\" for help page)");
            match self.input_handler.get() {
                Ok(InputValue::Enter) | Ok(InputValue::Yes) | Ok(InputValue::BtnCenter) => {
                    self.start_round()
                }
                Ok(InputValue::No) | Ok(InputValue::Quit) => break,
                Ok(InputValue::ToggleAi) => {
                    match self.ai_state {
                        AiState::Enabled => {
                            self.disable_ai();
                            println!("Toggling AI off");
                        }
                        AiState::Disabled => {
                            self.enable_ai();
                            println!("Toggling AI on");
                        }
                    };
                    self.players.set_active(ActivePlayer::Player1);
                }
                Ok(InputValue::ToggleButtons) => match self.button_state {
                    ButtonState::Enabled => {
                        self.disable_buttons();
                        println!("Toggling buttons off");
                    }
                    ButtonState::Disabled => {
                        self.enable_buttons().unwrap();
                        println!("Toggling buttons on");
                    }
                },
                Ok(InputValue::Help) => {
                    //? move all terminal output to a seperate file?
                    println!("Commands");
                    println!("  help\t\t\tshow this page");
                    println!("  toggle ai\t\ttoggle the ai on/off");
                    println!("  yes\t\t\tconfirm action (only when applicable)");
                    println!("  no\t\t\tconfirm action (only when applicable)");
                    println!("  KEY: Enter\t\tuse highlighted option (only when applicable)");
                    println!("  quit\t\t\tquit");

                    println!("Aliases");
                    println!("  h, ?\t\t\tshort for help");
                    println!("  ai\t\t\tshort for toggle ai");
                    println!("  y\t\t\tshort for yes");
                    println!("  n\t\t\tshort for no");
                    println!("  exit, stop, q, e, s\tshort for quit");
                }
                // Ok(InputValue::BtnUp)
                // | Ok(InputValue::BtnDown)
                // | Ok(InputValue::BtnLeft)
                // | Ok(InputValue::BtnRight) => (),
                _ => println!("Invalid"),
            }
        }
    }
}

impl<'g, const W: usize, const H: usize> Drop for Game<'g, W, H> {
    fn drop(&mut self) {
        self.board.reset();
        if self.display_state == DisplayState::Enabled {
            self.display.as_mut().unwrap().clear_animations();
        }
        self.update_disp();
        if let Some(disp) = self.display.take() {
            drop(disp.stop()); // drop to discard result without warning
        }
    }
}
