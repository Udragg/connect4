use c4_display::DisplayResult;
use rppal::gpio::{Gpio, InputPin, Level, Trigger};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};

// async

#[derive(Debug)]
pub(super) struct ButtonsAsync {
    // directions: [Direction; 5],
    pub up: ButtonAsync,
    pub down: ButtonAsync,
    pub left: ButtonAsync,
    pub right: ButtonAsync,
    pub center: ButtonAsync,
}

impl ButtonsAsync {
    pub(super) fn new(up: u8, down: u8, left: u8, right: u8, center: u8) -> DisplayResult<Self> {
        Ok(Self {
            up: ButtonAsync::new(up)?,
            down: ButtonAsync::new(down)?,
            left: ButtonAsync::new(left)?,
            right: ButtonAsync::new(right)?,
            center: ButtonAsync::new(center)?,
        })
    }
}

#[derive(Debug)]
pub(super) struct ButtonAsync {
    pub pin: InputPin,
    /// true = pressed, false = released
    pub last_state: bool,
    pub triggered: Arc<AtomicBool>,
    // pub last_trigger: Arc<Mutex<Instant>>,
}

impl ButtonAsync {
    fn new(pin: u8) -> DisplayResult<Self> {
        let mut pin = Gpio::new()?.get(pin)?.into_input_pullup();
        let last_state = match pin.read() {
            Level::High => false,
            Level::Low => true,
        };

        let triggered = Arc::new(AtomicBool::new(false));
        let last_trigger = Arc::new(Mutex::new(Instant::now()));

        let triggered_thread = triggered.clone();
        // let last_trigger_thread = last_trigger.clone();

        pin.set_async_interrupt(Trigger::FallingEdge, move |_| {
            if !triggered_thread.load(Ordering::SeqCst) {
                if last_trigger.lock().unwrap().elapsed() > Duration::from_millis(130) {
                    triggered_thread.store(true, Ordering::SeqCst);
                }
                let mut last_trigger = last_trigger.lock().unwrap();
                *last_trigger = Instant::now();
            }
        })
        .unwrap();

        Ok(Self {
            pin,
            last_state,
            triggered,
            // last_trigger,
        })
    }

    pub(super) fn read(&mut self) -> bool {
        self.triggered.swap(false, Ordering::SeqCst)
    }
}

// synchronous buttons (inferior)
#[allow(dead_code)]
mod sync_button {
    use std::time::{Duration, Instant};

    use c4_display::DisplayResult;
    use rppal::gpio::{Gpio, InputPin, Level};
    #[derive(Debug)]
    pub(super) struct Buttons {
        // directions: [Direction; 5],
        pub up: Button,
        pub down: Button,
        pub left: Button,
        pub right: Button,
        pub center: Button,
    }

    #[derive(Debug)]
    pub(super) struct Button {
        pub pin: InputPin,
        /// true = pressed, false = released
        pub last_state: bool,
        pub last_poll: Instant,
        pub last_change: Instant,
    }

    impl Buttons {
        pub(super) fn new(
            up: u8,
            down: u8,
            left: u8,
            right: u8,
            center: u8,
        ) -> DisplayResult<Self> {
            Ok(Self {
                up: Button::new(up)?,
                down: Button::new(down)?,
                left: Button::new(left)?,
                right: Button::new(right)?,
                center: Button::new(center)?,
            })
        }
    }

    impl Button {
        fn new(pin: u8) -> DisplayResult<Self> {
            let pin = Gpio::new()?.get(pin)?.into_input_pullup();
            let last_state = match pin.read() {
                Level::High => false,
                Level::Low => true,
            };
            let last_poll = Instant::now();
            let last_change = last_poll;

            Ok(Self {
                pin,
                last_state,
                last_poll,
                last_change,
            })
        }

        pub(super) fn read(&mut self) -> bool {
            let mut vals = [false; 11];

            for x in 0..11 {
                if self.pin.is_low() {
                    vals[x] = true;
                }
                c4_display::spin_wait(Duration::from_millis(2));
            }

            let mut counts = 0;

            for val in vals {
                if val {
                    counts += 1;
                } else {
                    counts -= 1;
                }
            }

            let state = counts > 0;

            self.last_poll = Instant::now();

            if state != self.last_state {
                self.last_change = self.last_poll;
                self.last_state = state;
            }

            state
        }
    }
}
