use crate::vulkan::input_state::KeyState::{Enter, Hold, Nothing, Release};
use glam::{vec2, I8Vec2, Vec2};
use winit::event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum KeyState {
    Nothing,
    Enter,
    Release,
    Hold,
}
impl KeyState {
    pub fn is_held(self) -> bool {
        Hold == self
    }
    pub fn is_down(self) -> bool {
        Hold == self || Enter == self
    }
    pub fn is_up(self) -> bool {
        Release == self || Nothing == self
    }
    pub fn is_released(self) -> bool {
        Release == self
    }
    pub fn is_entered(self) -> bool {
        Enter == self
    }
}

pub struct InputState {
    pub mouse_wheel_delta: I8Vec2,
    pub mouse_delta: Vec2,
    pub mouse_position: Vec2,
    pub mouse_left: KeyState,
    pub mouse_right: KeyState,
    pub mouse_middle: KeyState,
    pub mouse_on_screen: bool,

    pub key_w: KeyState,
    pub key_a: KeyState,
    pub key_s: KeyState,
    pub key_q: KeyState,
    pub key_e: KeyState,
    pub key_r: KeyState,
    pub key_z: KeyState,
    pub key_x: KeyState,
    pub key_c: KeyState,
    pub key_v: KeyState,
    pub key_esc: KeyState,
    pub key_space: KeyState,
    pub key1: KeyState,
    pub key2: KeyState,
    pub key3: KeyState,
    pub key4: KeyState,
    pub key5: KeyState,
    pub key6: KeyState,
    pub key7: KeyState,
    pub key8: KeyState,
    pub key9: KeyState,
    pub key0: KeyState,
    pub key_backspace: KeyState,

    pub key_d: KeyState,
    pub key_h: KeyState,
    pub key_alt: KeyState,
    pub key_ctrl: KeyState,
    pub key_shift: KeyState,
    pub key_enter: KeyState,
    pub key_left: KeyState,
    pub key_right: KeyState,
    pub key_up: KeyState,
    pub key_down: KeyState,
}

impl InputState {}

impl InputState {
    pub(crate) fn new() -> Self {
        Self::default()
    }
    pub fn set_mouse_delta(&mut self, x: f64, y: f64) -> () {
        self.mouse_delta = vec2(x as f32, x as f32);
    }
    fn set_mouse_position(&mut self, p: (f64, f64)) -> () {
        let (f1, f2) = p;
        self.mouse_position = vec2(f1 as f32, f2 as f32);
    }
    fn set_mouse_buttons(&mut self, _p: (f64, f64)) -> () {
        ();
    }

    pub fn read_event(&mut self, event: &WindowEvent) -> Option<()> {
        /*  if let Event::DeviceEvent {
            event: MouseMotion { delta, .. },
            ..
        } = event
        {
            // println!("Mouse delta: {:?} ", delta);
            self.set_mouse_delta(*delta)
        } else {
            self.set_mouse_delta((0.0, 0.0))
        }*/
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_delta = vec2(
                    position.x as f32 - self.mouse_position.x,
                    position.y as f32 - self.mouse_position.y,
                );
                self.set_mouse_position((position.x, position.y));
                //println!("Cursor moved {:?} ", position);
                Some(())
            }

            WindowEvent::MouseWheel { delta, .. } => match delta {
                MouseScrollDelta::LineDelta(_x, _y) => {
                    // println!("mouse wheel deltaxy: {}{}", x,y);
                    Some(())
                }
                _ => Some(()),
            },

            WindowEvent::MouseInput {
                device_id: _,
                state,
                button,
            } => {
                match button {
                    MouseButton::Left => {
                        self.mouse_left = InputState::set_mouse_key(&self.mouse_left, button, state)
                    }
                    MouseButton::Right => {
                        self.mouse_right =
                            InputState::set_mouse_key(&self.mouse_right, button, state)
                    }
                    MouseButton::Middle => {
                        self.mouse_middle =
                            InputState::set_mouse_key(&self.mouse_middle, button, state)
                    }
                    _ => {}
                }
                Some(())
            }

            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state,
                        physical_key,
                        ..
                    },
                ..
            } => {
                self.set_key_state(state, physical_key);
                Some(())
            }
            _ => None,
        }
    }
    pub fn set_key_state(&mut self, state: &ElementState, physical: &PhysicalKey) -> () {
        match physical {
            PhysicalKey::Code(key) => match key {
                KeyCode::AltLeft => {
                    self.key_alt = InputState::set_key(&self.key_alt, key, state);
                }
                KeyCode::AltRight => {
                    self.key_alt = InputState::set_key(&self.key_alt, key, state);
                }
                KeyCode::ShiftLeft => {
                    self.key_shift = InputState::set_key(&self.key_shift, key, state);
                }
                KeyCode::ShiftRight => {
                    self.key_shift = InputState::set_key(&self.key_shift, key, state);
                }
                KeyCode::KeyW => {
                    self.key_w = InputState::set_key(&self.key_w, key, state);
                }
                KeyCode::KeyA => {
                    self.key_a = InputState::set_key(&self.key_a, key, state);
                }
                KeyCode::KeyD => {
                    self.key_d = InputState::set_key(&self.key_d, key, state);
                }
                KeyCode::KeyQ => {
                    self.key_q = InputState::set_key(&self.key_q, key, state);
                }
                KeyCode::KeyE => {
                    self.key_e = InputState::set_key(&self.key_e, key, state);
                }
                KeyCode::KeyZ => {
                    self.key_z = InputState::set_key(&self.key_z, key, state);
                }
                KeyCode::KeyX => {
                    self.key_x = InputState::set_key(&self.key_x, key, state);
                }
                KeyCode::KeyC => {
                    self.key_c = InputState::set_key(&self.key_c, key, state);
                }
                KeyCode::KeyV => {
                    self.key_v = InputState::set_key(&self.key_v, key, state);
                }
                KeyCode::Escape => {
                    self.key_esc = InputState::set_key(&self.key_esc, key, state);
                }
                KeyCode::KeyS => {
                    self.key_s = InputState::set_key(&self.key_s, key, state);
                }
                KeyCode::Space => {
                    self.key_space = InputState::set_key(&self.key_space, key, state);
                }
                KeyCode::Digit0 => {
                    self.key0 = InputState::set_key(&self.key0, key, state);
                }
                KeyCode::Digit1 => {
                    self.key1 = InputState::set_key(&self.key1, key, state);
                }
                KeyCode::Digit2 => {
                    self.key2 = InputState::set_key(&self.key2, key, state);
                }
                KeyCode::Digit3 => {
                    self.key3 = InputState::set_key(&self.key3, key, state);
                }
                KeyCode::Digit4 => {
                    self.key4 = InputState::set_key(&self.key4, key, state);
                }
                KeyCode::Digit5 => {
                    self.key5 = InputState::set_key(&self.key5, key, state);
                }
                KeyCode::Digit6 => {
                    self.key6 = InputState::set_key(&self.key6, key, state);
                }
                KeyCode::Digit7 => {
                    self.key7 = InputState::set_key(&self.key7, key, state);
                }
                KeyCode::Digit8 => {
                    self.key8 = InputState::set_key(&self.key8, key, state);
                }
                KeyCode::Digit9 => {
                    self.key9 = InputState::set_key(&self.key9, key, state);
                }
                KeyCode::ArrowRight => {
                    self.key_right = InputState::set_key(&self.key_right, key, state);
                }
                KeyCode::ArrowLeft => {
                    self.key_left = InputState::set_key(&self.key_left, key, state);
                }
                KeyCode::ArrowUp => {
                    self.key_up = InputState::set_key(&self.key_up, key, state);
                }
                KeyCode::ArrowDown => {
                    self.key_down = InputState::set_key(&self.key_down, key, state);
                }
                KeyCode::Backspace => {
                    self.key_backspace = InputState::set_key(&self.key_backspace, key, state);
                }

                a => println!("{:?} {:?}", a, state),
            },
            _ => {}
        }
    }

    fn set_key(current_state: &KeyState, _l_key: &KeyCode, input_state: &ElementState) -> KeyState {
        match input_state {
            ElementState::Released => {
                //println!("Releasing {:?}", l_key);
                Release
            }
            ElementState::Pressed => match current_state {
                Release => {
                    //println!("Entering {:?}", l_key);
                    Enter
                }
                Enter => {
                    //println!("Holding {:?}", l_key);
                    Hold
                }
                Hold => {
                    //println!("Holding {:?}", l_key);
                    Hold
                }
                Nothing => {
                    //println!("Entering {:?}", l_key);
                    Enter
                }
            },
        }
    }
    fn set_mouse_key(
        current_state: &KeyState,
        _l_key: &MouseButton,
        input_state: &ElementState,
    ) -> KeyState {
        match input_state {
            ElementState::Released => {
                //println!("Releasing {:?}", l_key);
                Release
            }
            ElementState::Pressed => match current_state {
                Release => {
                    //println!("Entering {:?}", l_key);
                    Enter
                }
                Enter => {
                    //println!("Holding {:?}", l_key);
                    Hold
                }
                Hold => {
                    //println!("Holding {:?}", l_key);
                    Hold
                }
                Nothing => {
                    //println!("Entering {:?}", l_key);
                    Enter
                }
            },
        }
    }
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            mouse_wheel_delta: I8Vec2::new(0, 0),
            mouse_delta: vec2(0.0, 0.0),
            mouse_position: vec2(0.0, 0.0),
            mouse_left: KeyState::Nothing,
            mouse_right: KeyState::Nothing,
            mouse_middle: KeyState::Nothing,
            mouse_on_screen: false,
            key_w: KeyState::Nothing,
            key_a: KeyState::Nothing,
            key_s: KeyState::Nothing,
            key_q: KeyState::Nothing,
            key_e: KeyState::Nothing,
            key_r: KeyState::Nothing,
            key_z: KeyState::Nothing,
            key_x: KeyState::Nothing,
            key_c: KeyState::Nothing,
            key_v: KeyState::Nothing,
            key_esc: KeyState::Nothing,
            key_space: KeyState::Nothing,
            key1: KeyState::Nothing,
            key2: KeyState::Nothing,
            key3: KeyState::Nothing,
            key4: KeyState::Nothing,
            key5: KeyState::Nothing,
            key6: KeyState::Nothing,
            key7: KeyState::Nothing,
            key8: KeyState::Nothing,
            key9: KeyState::Nothing,
            key0: KeyState::Nothing,
            key_d: KeyState::Nothing,
            key_h: KeyState::Nothing,
            key_alt: KeyState::Nothing,
            key_ctrl: KeyState::Nothing,
            key_shift: KeyState::Nothing,
            key_enter: KeyState::Nothing,
            key_right: KeyState::Nothing,
            key_up: KeyState::Nothing,
            key_backspace: KeyState::Nothing,
            key_left: KeyState::Nothing,
            key_down: KeyState::Nothing,
        }
    }
}
