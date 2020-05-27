use device_query::{DeviceQuery, DeviceState, Keycode};

pub struct EventListener {
    device_state: DeviceState,
    last_pressed_keys: Vec<Keycode>,
}

#[derive(Hash, Eq, PartialEq, Debug)]
pub enum KeyEvent {
    Press,
    Release,
}

impl Default for EventListener {
    fn default() -> Self {
        let device_state = DeviceState::new();
        let last_pressed_keys = device_state.get_keys();
        Self {
            device_state,
            last_pressed_keys,
        }
    }
}

impl EventListener {
    pub fn get_events(&mut self) -> Vec<(Keycode, KeyEvent)> {
        let pressed_keys = self.device_state.get_keys();
        let mut res = vec![];
        for press in &pressed_keys {
            if !self.last_pressed_keys.contains(press) {
                res.push((press.clone(), KeyEvent::Press));
            }
        }
        for release in &self.last_pressed_keys {
            if !pressed_keys.contains(release) {
                res.push((release.clone(), KeyEvent::Release));
            }
        }
        self.last_pressed_keys = pressed_keys;
        res
    }
}
