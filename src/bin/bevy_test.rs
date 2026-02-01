use std::f32::consts::PI;

use VulcanEngine_0::{
    bevy_app,
    game_objects::{skybox::SkyBox, transform::Transform},
    vulkan::winit_render_app,
};
use bevy::{
    app::{App, PostStartup, Startup},
    ecs::system::ResMut,
};
use glam::{Quat, Vec3};

pub fn main() {
    App::new()
        .add_plugins(bevy_app::app::VulkanDefault)
        .add_systems(PostStartup, add_objects)
        // Bevy itself:
        // ...
        // launch the app!
        .run();
}

fn add_objects(mut app: ResMut<winit_render_app::App>) {
    let _paths = [
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
                    rotation: Quat::from_rotation_y(-PI / 2.0),
                },
            )
            .unwrap();
    }
    let ashtray = app.add_object("assets/living_room/Chair.glb").unwrap();
    for a in ashtray {
        app.scene
            .transform_object(
                a,
                Transform {
                    position: Vec3::ZERO,
                    scale: 4.0 * Vec3::ONE,
                    rotation: Quat::from_rotation_y(PI),
                },
            )
            .unwrap();
    }
    let man = app.add_object("assets/LittleMan.glb").unwrap();
    let man = man.iter().next().unwrap();

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
    //let building = app.add_object("assets/city_building.glb").unwrap();
    let skybox = Some(app.load_skybox());
    app.scene.skybox = skybox;

    /*app.scene.skybox = Some(
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
    )*/
}
