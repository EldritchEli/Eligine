use std::f32::consts::PI;

use VulcanEngine_0::{
    game_objects::{skybox::SkyBox, transform::Transform},
    vulkan::renderer::{self, VulkanData},
};
use glam::{Quat, Vec3};

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

    vulkan_data
        .set_init(|app| {
            let paths = [
                //"assets/bird_orange.glb",
                //"assets/living_room/Rubiks Cube.glb",
                "assets/LittleMan.glb",
                "assets/PlatformerCharacter.glb",
            ];

            let guy = app.add_object("assets/PlatformerCharacter.glb").unwrap();
            for g in guy {
                app.scene
                    .transform_object(
                        g,
                        Transform {
                            position: Vec3::new(7.0, -2.0, 6.0),
                            scale: 4.0 * Vec3::ONE,
                            rotation: Quat::default(),
                        },
                    )
                    .unwrap();
            }
            let man = app.add_object("assets/LittleMan.glb").unwrap();
            let man = man.iter().next().unwrap();
            let ashtray = app.add_object("assets/living_room/Chair.glb").unwrap();
            let ashtray = ashtray.iter().next().unwrap();

            app.scene
                .transform_object(
                    *man,
                    Transform {
                        position: Vec3::new(-0.0, -2.0, 8.0),
                        scale: 8.0 * Vec3::ONE,
                        rotation: Quat::default(),
                    },
                )
                .unwrap();
            let building = app.add_object("assets/city_building.glb").unwrap();
            /*            app.scene.skybox = Some(
                SkyBox::load(
                    &app.instance,
                    &app.device,
                    &mut app.data,
                    "assets/skyboxes/pretty_sky/py.png",
                    "assets/skyboxes/pretty_sky/ny.png",
                    "assets/skyboxes/pretty_sky/pz.png",
                    "assets/skyboxes/pretty_sky/nz.png",
                    "assets/skyboxes/pretty_sky/nx.png",
                    "assets/skyboxes/pretty_sky/px.png",
                )
                .unwrap(),
            )*/

            app.scene.skybox = Some(
                SkyBox::load(
                    &app.instance,
                    &app.device,
                    &mut app.data,
                    "assets/skyboxes/nebula/top.png",
                    "assets/skyboxes/nebula/bottom.png",
                    "assets/skyboxes/nebula/left.png",
                    "assets/skyboxes/nebula/right.png",
                    "assets/skyboxes/nebula/back.png",
                    "assets/skyboxes/nebula/front.png",
                )
                .unwrap(),
            )
        })
        .unwrap();
    event_loop.run_app(&mut vulkan_data).map_err(OneOf::new)
}
