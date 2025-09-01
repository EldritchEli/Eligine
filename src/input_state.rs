use crate::input_state::KeyState::{Enter, Hold, Nothing, Release};
use nalgebra_glm::{vec2, I8Vec2, Vec2};
use winit::event::DeviceEvent::MouseMotion;
use winit::event::WindowEvent::KeyboardInput;
use winit::event::{ElementState, Event, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent};
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

    pub keyW: KeyState,
    pub keyA: KeyState,
    pub keyS: KeyState,
    pub keyQ: KeyState,
    pub keyE: KeyState,
    pub keyR: KeyState,
    pub keyZ: KeyState,
    pub keyX: KeyState,
    pub keyC: KeyState,
    pub keyV: KeyState,
    pub keyEsc: KeyState,
    pub keySpace: KeyState,
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
    pub keyBackspace: KeyState,

    pub keyD: KeyState,
    pub keyH: KeyState,
    pub keyAlt: KeyState,
    pub keyCtrl: KeyState,
    pub keyShift: KeyState,
    pub keyEnter: KeyState,
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
    pub fn set_mouse_delta(&mut self, p0: (f64, f64)) -> () {
        let (f1, f2) = p0;
        self.mouse_delta = vec2(f1 as f32, f2 as f32);
    }
    fn set_mouse_position(&mut self, p: (f64, f64)) -> () {
        let (f1, f2) = p;
        self.mouse_position = vec2(f1 as f32, f2 as f32);
    }
    fn set_mouse_buttons(&mut self, p: (f64, f64)) -> () {
        ();
    }

    pub fn read_event(&mut self, event: &winit::event::Event<()>) -> Option<()> {
        if let Event::DeviceEvent {
            event: MouseMotion { delta, .. },
            ..
        } = event
        {
            // println!("Mouse delta: {:?} ", delta);
            self.set_mouse_delta(*delta)
        } else {
            self.set_mouse_delta((0.0, 0.0))
        }
        match event {
            Event::WindowEvent {
                event: WindowEvent::CursorMoved { position, .. },
                ..
            } => {
                self.set_mouse_position((position.x, position.y));
                //println!("Cursor moved {:?} ", position);
                Some(())
            }

            Event::WindowEvent {
                event: WindowEvent::MouseWheel { delta, .. },
                ..
            } => match delta {
                MouseScrollDelta::LineDelta(x, y) => {
                    // println!("mouse wheel deltaxy: {}{}", x,y);
                    Some(())
                }
                _ => Some(()),
            },
            Event::WindowEvent {
                event:
                    WindowEvent::MouseInput {
                        device_id,
                        state,
                        button,
                    },
                ..
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
            Event::WindowEvent {
                event:
                    KeyboardInput {
                        event:
                            KeyEvent {
                                state,
                                physical_key,
                                ..
                            },
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
                    self.keyAlt = InputState::set_key(&self.keyAlt, key, state);
                }
                KeyCode::AltRight => {
                    self.keyAlt = InputState::set_key(&self.keyAlt, key, state);
                }
                KeyCode::ShiftLeft => {
                    self.keyShift = InputState::set_key(&self.keyShift, key, state);
                }
                KeyCode::ShiftRight => {
                    self.keyShift = InputState::set_key(&self.keyShift, key, state);
                }
                KeyCode::KeyW => {
                    self.keyW = InputState::set_key(&self.keyW, key, state);
                }
                KeyCode::KeyA => {
                    self.keyA = InputState::set_key(&self.keyA, key, state);
                }
                KeyCode::KeyD => {
                    self.keyD = InputState::set_key(&self.keyD, key, state);
                }
                KeyCode::KeyQ => {
                    self.keyQ = InputState::set_key(&self.keyQ, key, state);
                }
                KeyCode::KeyE => {
                    self.keyE = InputState::set_key(&self.keyE, key, state);
                }
                KeyCode::KeyZ => {
                    self.keyZ = InputState::set_key(&self.keyZ, key, state);
                }
                KeyCode::KeyX => {
                    self.keyX = InputState::set_key(&self.keyX, key, state);
                }
                KeyCode::KeyC => {
                    self.keyC = InputState::set_key(&self.keyC, key, state);
                }
                KeyCode::KeyV => {
                    self.keyV = InputState::set_key(&self.keyV, key, state);
                }
                KeyCode::Escape => {
                    self.keyEsc = InputState::set_key(&self.keyEsc, key, state);
                }
                KeyCode::KeyS => {
                    self.keyS = InputState::set_key(&self.keyS, key, state);
                }
                KeyCode::Space => {
                    self.keySpace = InputState::set_key(&self.keySpace, key, state);
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
                    self.keyBackspace = InputState::set_key(&self.keyBackspace, key, state);
                }

                a => println!("{:?} {:?}", a, state),
            },
            _ => {}
        }
    }

    fn set_key(current_state: &KeyState, l_key: &KeyCode, input_state: &ElementState) -> KeyState {
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
        l_key: &MouseButton,
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
            keyW: KeyState::Nothing,
            keyA: KeyState::Nothing,
            keyS: KeyState::Nothing,
            keyQ: KeyState::Nothing,
            keyE: KeyState::Nothing,
            keyR: KeyState::Nothing,
            keyZ: KeyState::Nothing,
            keyX: KeyState::Nothing,
            keyC: KeyState::Nothing,
            keyV: KeyState::Nothing,
            keyEsc: KeyState::Nothing,
            keySpace: KeyState::Nothing,
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
            keyD: KeyState::Nothing,
            keyH: KeyState::Nothing,
            keyAlt: KeyState::Nothing,
            keyCtrl: KeyState::Nothing,
            keyShift: KeyState::Nothing,
            keyEnter: KeyState::Nothing,
            key_right: KeyState::Nothing,
            key_up: KeyState::Nothing,
            keyBackspace: KeyState::Nothing,
            key_left: KeyState::Nothing,
            key_down: KeyState::Nothing,
        }
    }
}
