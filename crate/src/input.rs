use std::collections::{HashMap, HashSet, VecDeque};
use super::Key;

pub enum InputEvent {
    Press(Key),
    Release(Key),
    MouseMovement(usize, usize, isize, isize),
    MouseClick(usize, usize),
}

pub struct InputEvents {
    pub events: VecDeque<InputEvent>,
}

impl InputEvents {
    pub fn new() -> Self {
        InputEvents {
            events: VecDeque::with_capacity(32),
        }
    }
}

#[derive(Default)] 
pub struct MouseState {
    pub position: (usize, usize),
    //pub dragging_from: Option<(usize, usize)>,
}

pub struct InputState {
    pub key_held: HashSet<Key>,
    pub key_pressed: HashMap<Key, i32>,
    pub mouse: MouseState,
}

impl InputState {
    pub fn new() -> Self {
        InputState {
            key_held: HashSet::with_capacity(16),
            key_pressed: HashMap::with_capacity(16),
            mouse: MouseState::default(),
        }
    }

    // FIXME: The fact that this mutates InputState is surprising; think of another name.
    // Maybe handle_button, pop_button, pop_press_event_or_held.
    pub fn key_pressed_or_held(&mut self, key: &Key) -> bool {
        if let Some(_press_count) = self.key_pressed.remove(&key) {
            return true;
        } else {
            return self.key_held.contains(&key);
        }
    }

    pub fn key_pressed(&mut self, key: &Key) -> bool {
        self.key_pressed.remove(&key).is_some()
    }
}
