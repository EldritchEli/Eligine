use std::f32::consts::PI;

use glam::{Quat, Vec3};
use VulcanEngine_0::{
    game_objects::transform::Transform,
    vulkan::renderer::{self, VulkanData},
};

use terrors::OneOf;
use winit::{
    error::{EventLoopError, OsError},
    event_loop::{self, ControlFlow},
};

use vulkanalia::vk::ErrorCode;
fn main() -> Result<(), OneOf<(OsError, anyhow::Error, EventLoopError, ErrorCode)>> {
    let event_loop = event_loop::EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut vulkan_data = VulkanData::default();
    //let mut vulkan_data = renderer::init("Eligine").map_err(OneOf::broaden)?;
    /* let paths = [
         "assets/city_building.glb",
         "assets/bird_orange.glb",
         "assets/living_room/Rubiks Cube.glb",
         //"assets/Platformer/Character/glTF/Character.gltf",
     ];
     let (_object_keys, _render_keys) = vulkan_data.app.add_render_objects(&paths, true);
    */
    event_loop.run_app(&mut vulkan_data).map_err(OneOf::new)
}
