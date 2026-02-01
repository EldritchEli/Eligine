use bevy::ecs::component::Component;
use glam::{Mat4, Quat, Vec3};
use std::{f32::consts::PI, ops::Mul};

#[derive(Debug, PartialEq, Clone, Component)]
pub struct Transform {
    pub position: Vec3,
    pub scale: Vec3,
    pub rotation: Quat,
}
impl Mul for Transform {
    type Output = Transform;

    fn mul(self, rhs: Self) -> Self::Output {
        Transform::new(
            self.position + rhs.position,
            self.scale * rhs.scale,
            rhs.rotation * self.rotation,
        )
    }
}
impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            scale: Vec3::ONE,
            rotation: Quat::from_rotation_x(PI / 2.0),
        }
    }
}

impl Transform {
    pub fn new(position: Vec3, scale: Vec3, rotation: Quat) -> Self {
        Transform {
            position,
            scale,
            rotation,
        }
    }
    pub fn from_position(position: Vec3) -> Self {
        Self {
            position,
            ..Default::default()
        }
    }
    pub fn from_rotation(rotation: Quat) -> Self {
        Self {
            rotation: Quat::from_rotation_x(PI / 2.0) * rotation,
            ..Default::default()
        }
    }
    pub fn from_position_rotation(position: Vec3, rotation: Quat) -> Self {
        Self {
            position,
            rotation,
            ..Default::default()
        }
    }

    pub fn matrix(&self) -> Mat4 {
        Mat4::from_translation(self.position)
            * Mat4::from_quat(self.rotation)
            * Mat4::from_scale(self.scale)
    }
}
