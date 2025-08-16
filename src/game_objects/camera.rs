use std::cmp::PartialEq;
use std::ops::Add;
use nalgebra::Quaternion;
use nalgebra_glm::{identity, look_at_rh, mat4, quat_euler_angles, quat_look_at_rh, quat_rotate, quat_rotate_normalized_axis, sin, translate, translation, vec3, Mat4, Qua, Quat, Vec3};
use crate::input_state::{InputState, KeyState};

#[derive(PartialEq, Copy, Clone, Debug)]
pub struct Camera {
  movement_speed: f32,
  rotation_speed: f32,
  zoom_speed: f32,
  fov: f32,
  pub position: Vec3,
  pub euler: Vec3,
  pub rotation: Qua<f32>
}



impl Camera {
  pub fn new(position: Vec3, orientation: Vec3, movement_speed : f32,
             rotation_speed : f32, zoom_speed : f32,fov: f32) -> Self {
    let quat = quat_look_at_rh(&position,&(position+orientation));
    Self { movement_speed, rotation_speed, zoom_speed, fov, position, euler : vec3(0.0,0.0,0.0), rotation: Qua::identity() }}
  pub fn set_movement_speed(&mut self, movement_speed: f32) {
        self.movement_speed = movement_speed; }
  pub fn set_rotation_speed(&mut self, rotation_speed: f32) {
        self.rotation_speed = rotation_speed; }
  pub fn set_zoom_speed(&mut self, zoom_speed: f32) {
        self.zoom_speed = zoom_speed; }
  pub fn set_fov(&mut self, fov: f32) { self.fov = fov; }

  pub fn rotate_xy(&mut self, y: f32, x: f32) {
    self.euler.x += x*self.rotation_speed*0.01;
    self.euler.y += y*self.rotation_speed*0.01;
    let yawed = quat_rotate_normalized_axis(&Quaternion::identity(),self.euler.x,&vec3(0.0,1.0,0.0));
    self.rotation = quat_rotate_normalized_axis(&yawed,self.euler.y,&vec3(1.0,0.0,0.0));


  }
  pub fn move_forward(&mut self, amount : f32) {
    let v : Vec3 =  (amount*self.movement_speed)*self.direction();
    self.position += v;
  }
  pub fn move_right(&mut self, amount: f32) {
    let v : Vec3 = (amount*self.movement_speed)*self.bidirection();
    self.position += v;
  }
  pub fn move_up(&mut self, amount: f32) {
    let v : Vec3 = (amount*self.movement_speed)*self.bidirection_up();
    self.position += v;
  }
  pub fn matrix(&self) -> Mat4 {
    let looking_at = self.position + self.direction();

    look_at_rh(&self.position, &looking_at, &vec3(0.0, 1.0, 0.0))

  }
  pub fn direction(&self) -> Vec3 {
    (self.rotation*
      Qua::new(0.0,0.0,0.0,-1.0)*
      self.rotation.conjugate()).imag()
  }
  pub fn bidirection(&self) -> Vec3  {
    (self.rotation*
      Qua::new(0.0,1.0,0.0,0.0)*
      self.rotation.conjugate()).imag()
  }
  pub fn bidirection_up(&self) -> Vec3 {
    (self.rotation*
      Qua::new(0.0,0.0,1.0,0.0)*
    self.rotation.conjugate()).imag()
  }

  pub fn update(&mut self, delta_time: f32, input : &InputState) {
    let mouse_delta  = input.mouse_delta;

    if input.keyW.is_down() {
      self.move_forward(delta_time);
    };
    if input.keyS.is_down() {
      self.move_forward(-delta_time);
    };

    if input.keyA.is_down() { self.move_right(delta_time ); };
    if input.keyD.is_down() { self.move_right(-delta_time ); };
    if input.keyE.is_down() { self.move_up(-delta_time ); };
    if input.keyQ.is_down() { self.move_up(delta_time ); };



    if input.mouse_right.is_down() {
      self.rotate_xy(mouse_delta.x, mouse_delta.y);
      //println!("pitch: {}, yaw: {}, position: {:?}, quaternion: {:?} ", self.pitch, self.yaw, self.position, self.quaternion());
    }
  }
}
impl Default for Camera {
  fn default() -> Self {
    let pos = vec3(0.0,0.0,0.0);
    Self::new(pos,
              -pos,
              1.0,
              0.20,
              1.0,
              45.0)
  }
}

