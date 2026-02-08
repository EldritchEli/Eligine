use crate::vulkan::input_state::KeyState::{Enter, Hold, Nothing, Release};
use bevy::ecs::resource::Resource;
use glam::{Vec2, vec2};
use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, KeyEvent, MouseButton, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

#[derive(Default, PartialEq, Debug, Clone, Copy)]
pub enum KeyState {
    #[default]
    Nothing,
    Enter,
    Release,
    Hold,
}
impl KeyState {
    pub fn is_held(&self) -> bool {
        Hold == *self
    }
    pub fn is_down(&self) -> bool {
        Hold == *self || Enter == *self
    }
    pub fn is_up(&self) -> bool {
        Release == *self || Nothing == *self
    }
    pub fn is_released(&self) -> bool {
        Release == *self
    }
    pub fn is_entered(&mut self) -> bool {
        let entered = Enter == *self;
        if entered {
            *self = Hold;
        }
        entered
    }
}
#[derive(Debug, Clone, Default, Resource)]
pub struct InputState {
    pub mouse_wheel_delta: Vec2,
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
    pub f1: KeyState,
    pub f2: KeyState,
    pub f3: KeyState,
    pub f4: KeyState,
    pub f5: KeyState,
    pub f6: KeyState,
    pub f7: KeyState,
    pub f8: KeyState,
    pub f9: KeyState,
    pub f10: KeyState,
    pub f11: KeyState,
    pub f12: KeyState,
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

impl InputState {
    pub fn set_mouse(&mut self, position: &PhysicalPosition<f64>) {
        self.mouse_delta = vec2(
            position.x as f32 - self.mouse_position.x,
            position.y as f32 - self.mouse_position.y,
        );
        self.mouse_position = vec2(position.x as f32, position.y as f32);
    }
    pub fn reset_mouse_delta(&mut self) {
        self.mouse_delta = Vec2::ZERO
    }

    pub fn read_event(&mut self, event: &WindowEvent) {
        //self.set_mouse_position(event);
        //self.set_mouse_scroll_delta(event);
        match event {
            WindowEvent::MouseInput {
                device_id: _,
                state,
                button,
            } => match button {
                MouseButton::Left => {
                    self.mouse_left = InputState::set_mouse_key(&self.mouse_left, button, state)
                }
                MouseButton::Right => {
                    self.mouse_right = InputState::set_mouse_key(&self.mouse_right, button, state)
                }
                MouseButton::Middle => {
                    self.mouse_middle = InputState::set_mouse_key(&self.mouse_middle, button, state)
                }
                _ => {}
            },
            WindowEvent::CursorMoved { position, .. } => self.set_mouse(position),
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state,
                        physical_key: PhysicalKey::Code(key),
                        repeat,
                        ..
                    },
                ..
            } => {
                if !repeat {
                    self.set_key_state(state, key);
                }
            }
            _ => (),
        }
    }
    pub fn set_key_state(&mut self, state: &ElementState, key: &KeyCode) {
        match key {
            KeyCode::AltLeft => {
                self.key_alt = InputState::set_key(state);
            }
            KeyCode::AltRight => {
                self.key_alt = InputState::set_key(state);
            }
            KeyCode::ShiftLeft => {
                self.key_shift = InputState::set_key(state);
            }
            KeyCode::ShiftRight => {
                self.key_shift = InputState::set_key(state);
            }
            KeyCode::KeyW => {
                self.key_w = InputState::set_key(state);
            }
            KeyCode::KeyA => {
                self.key_a = InputState::set_key(state);
            }
            KeyCode::KeyD => {
                self.key_d = InputState::set_key(state);
            }
            KeyCode::KeyQ => {
                self.key_q = InputState::set_key(state);
            }
            KeyCode::KeyE => {
                self.key_e = InputState::set_key(state);
            }
            KeyCode::KeyZ => {
                self.key_z = InputState::set_key(state);
            }
            KeyCode::KeyX => {
                self.key_x = InputState::set_key(state);
            }
            KeyCode::KeyC => {
                self.key_c = InputState::set_key(state);
            }
            KeyCode::KeyV => {
                self.key_v = InputState::set_key(state);
            }
            KeyCode::Escape => {
                self.key_esc = InputState::set_key(state);
            }
            KeyCode::KeyS => {
                self.key_s = InputState::set_key(state);
            }
            KeyCode::Space => {
                self.key_space = InputState::set_key(state);
            }
            KeyCode::Digit0 => {
                self.key0 = InputState::set_key(state);
            }
            KeyCode::Digit1 => {
                self.key1 = InputState::set_key(state);
            }
            KeyCode::Digit2 => {
                self.key2 = InputState::set_key(state);
            }
            KeyCode::Digit3 => {
                self.key3 = InputState::set_key(state);
            }
            KeyCode::Digit4 => {
                self.key4 = InputState::set_key(state);
            }
            KeyCode::Digit5 => {
                self.key5 = InputState::set_key(state);
            }
            KeyCode::Digit6 => {
                self.key6 = InputState::set_key(state);
            }
            KeyCode::Digit7 => {
                self.key7 = InputState::set_key(state);
            }
            KeyCode::Digit8 => {
                self.key8 = InputState::set_key(state);
            }
            KeyCode::Digit9 => {
                self.key9 = InputState::set_key(state);
            }
            KeyCode::ArrowRight => {
                self.key_right = InputState::set_key(state);
            }
            KeyCode::ArrowLeft => {
                self.key_left = InputState::set_key(state);
            }
            KeyCode::ArrowUp => {
                self.key_up = InputState::set_key(state);
            }
            KeyCode::ArrowDown => {
                self.key_down = InputState::set_key(state);
            }
            KeyCode::Backspace => {
                self.key_backspace = InputState::set_key(state);
            }
            KeyCode::F1 => self.f1 = InputState::set_key(state),
            KeyCode::F2 => self.f2 = InputState::set_key(state),
            KeyCode::F3 => self.f3 = InputState::set_key(state),
            KeyCode::F4 => self.f4 = InputState::set_key(state),
            KeyCode::F5 => self.f5 = InputState::set_key(state),
            KeyCode::F6 => self.f6 = InputState::set_key(state),
            KeyCode::F7 => self.f7 = InputState::set_key(state),
            KeyCode::F8 => self.f8 = InputState::set_key(state),
            KeyCode::F9 => self.f9 = InputState::set_key(state),
            KeyCode::F10 => self.f10 = InputState::set_key(state),
            KeyCode::F11 => self.f11 = InputState::set_key(state),
            KeyCode::F12 => self.f12 = InputState::set_key(state),
            _ => {}
        }
    }

    fn set_key(input_state: &ElementState) -> KeyState {
        match input_state {
            ElementState::Released => {
                //println!("Releasing {:?}", l_key);
                Release
            }
            ElementState::Pressed => Enter,
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
