use std::{
    str::FromStr,
    sync::mpsc::{channel, Receiver, SendError, Sender, TryRecvError},
    thread,
    time::{Duration, Instant},
};

use crate::game::error::{Error, GameResult};

use super::button::ButtonsAsync;

#[derive(Debug)]
pub(super) struct InputHandler {
    stdin_rx: Receiver<(GameResult<InputValue>, Instant)>,
    btn_rx: Option<Receiver<(InputValue, Instant)>>,
    btn_tx: Option<Sender<Command>>,
    _remote_rx: Option<Receiver<(InputValue, Instant)>>,
    _remote_tx: Option<Sender<Command>>,
    buf: Vec<(GameResult<InputValue>, Instant)>,
}

impl InputHandler {
    pub(super) fn new() -> Self {
        // communication from thread
        let (stdin_tx_int, stdin_rx_ext) = channel::<(GameResult<InputValue>, Instant)>();

        thread::spawn(move || loop {
            match stdin_tx_int.send((InputValue::get(), Instant::now())) {
                Ok(()) => (),
                Err(_) => break,
            }
        });

        Self {
            stdin_rx: stdin_rx_ext,
            btn_tx: None,
            btn_rx: None,
            _remote_tx: None,
            _remote_rx: None,
            buf: Vec::new(),
        }
    }

    pub(super) fn get(&mut self) -> GameResult<InputValue> {
        self.buf.clear();
        loop {
            if let Some(rx) = &self._remote_rx {
                while let Ok(input) = rx.try_recv() {
                    let (input, time) = input;
                    self.buf.push((Ok(input), time));
                }
            }

            if let Some(rx) = &self.btn_rx {
                while let Ok(input) = rx.try_recv() {
                    let (input, time) = input;
                    self.buf.push((Ok(input), time));
                }
            }

            while let Ok(input) = self.stdin_rx.try_recv() {
                self.buf.push(input);
            }

            self.buf
                .retain(|input| input.1 + Duration::from_millis(200) > Instant::now());

            if let Some(input) = self.buf.pop() {
                return input.0;
            }

            std::thread::sleep(Duration::from_millis(1));
        }
    }

    /// Start receiving inputs from buttons. Requires button pin numbers.
    pub(super) fn start_buttons(
        &mut self,
        btn_up: u8,
        btn_down: u8,
        btn_left: u8,
        btn_right: u8,
        btn_center: u8,
    ) -> GameResult<()> {
        // communication to thread
        let (btn_tx_ext, btn_rx_int) = channel::<Command>();

        // communication from thread
        let (btn_tx_int, btn_rx_ext) = channel::<(InputValue, Instant)>();

        let mut buttons = ButtonsAsync::new(btn_up, btn_down, btn_left, btn_right, btn_center)?;

        thread::spawn(move || loop {
            match btn_rx_int.try_recv() {
                Err(TryRecvError::Disconnected) => break,
                Err(TryRecvError::Empty) => (),
                Ok(cmd) => match cmd {
                    Command::Stop => break,
                },
            }

            if buttons.up.read() {
                btn_tx_int
                    .send((InputValue::BtnUp, Instant::now()))
                    .expect("Failed to send message");
            }

            if buttons.down.read() {
                btn_tx_int
                    .send((InputValue::BtnDown, Instant::now()))
                    .expect("Failed to send message");
            }

            if buttons.left.read() {
                btn_tx_int
                    .send((InputValue::BtnLeft, Instant::now()))
                    .expect("Failed to send message");
            }

            if buttons.right.read() {
                btn_tx_int
                    .send((InputValue::BtnRight, Instant::now()))
                    .expect("Failed to send message");
            }

            if buttons.center.read() {
                btn_tx_int
                    .send((InputValue::BtnCenter, Instant::now()))
                    .expect("Failed to send message");
            }

            std::thread::sleep(std::time::Duration::from_millis(20));
        });

        self.btn_tx = Some(btn_tx_ext);
        self.btn_rx = Some(btn_rx_ext);

        Ok(())
    }

    /// Stop receiving input from buttons. Resets gpio pins used by buttons.
    pub(super) fn stop_buttons(&mut self) {
        if let Some(tx) = self.btn_tx.take() {
            tx.send(Command::Stop).expect("Failed to send command");
        }

        self.btn_rx.take();
    }
}

impl Drop for InputHandler {
    fn drop(&mut self) {
        self.stop_buttons();
        // self.stop_remote();
    }
}

enum Command {
    Stop,
}

/// The different types of io input the game can ask for.
#[derive(Debug, Clone, Copy)]
pub(super) enum InputValue {
    Col(usize),
    Enter,
    Yes,
    No,
    Quit,
    ToggleAi,
    Help,
    ToggleButtons,
    BtnUp,
    BtnDown,
    BtnLeft,
    BtnRight,
    BtnCenter,
    // AiAdvice, // TODO ask the ai for placement advice (also useful for ai debugging)
}

impl InputValue {
    /// Attempt to get input from stdin.
    fn get() -> GameResult<Self> {
        let mut buf = String::new();

        std::io::stdin()
            .read_line(&mut buf)
            .expect("Failed to read stdin");
        InputValue::from_str(&buf)
    }
}

impl std::str::FromStr for InputValue {
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
            col if col.parse::<usize>().is_ok() => Ok(Self::Col(col.parse::<usize>().unwrap())),
            str => Err(Error::InvalidInput(str.to_string())),
        }
    }
}
