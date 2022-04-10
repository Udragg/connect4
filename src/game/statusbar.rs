// use super::components::Player;

#[derive(Debug, Default)]
pub(super) struct StatusBar<const W: usize> {
    // pub bar: [Player; W],
    pub selected_pos: usize,
}

impl<const W: usize> StatusBar<W> {
    pub(super) fn selection_left(&mut self) {
        self.selected_pos += W;
        self.selected_pos -= 1;
        self.selected_pos %= W;
    }

    pub(super) fn selection_right(&mut self) {
        self.selected_pos += 1;
        self.selected_pos %= W;
    }

    pub(super) fn selection_set(&mut self, pos: usize) {
        self.selected_pos = pos;
        self.selected_pos %= W;
    }
}

// impl<const W: usize> Default for StatusBar {}
