use std::f32::consts::PI;

use VulcanEngine_0::{
    asset_manager,
    bevy_app::{self, render::VulkanApp},
    game_objects::{scene::Scene, skybox::SkyBox, transform::Transform},
    vulkan::winit_render_app::AppData,
};
use bevy::{
    app::{App, PostStartup},
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

fn add_objects(mut app: ResMut<VulkanApp>, mut data: ResMut<AppData>, mut scene: ResMut<Scene>) {
    const paths: [&str; 2] = [
        //"assets/bird_orange.glb",
        //"assets/living_room/Rubiks Cube.glb",
        "assets/LittleMan.glb",
        "assets/PlatformerCharacter.glb",
    ];
    let guy = asset_manager::load::scene(
        &app.instance,
        &app.device,
        &mut data,
        &mut scene,
        "assets/PlatformerCharacter.glb",
    )
    .unwrap();
    for g in guy {
        scene
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
    /*let ashtray = app.add_object("assets/living_room/Chair.glb").unwrap();
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
    }*/
    let man = asset_manager::load::scene(
        &app.instance,
        &app.device,
        &mut data,
        &mut scene,
        "assets/LittleMan.glb",
    )
    .unwrap();

    scene
        .transform_object(
            man[0],
            Transform {
                position: Vec3::new(-0.0, -2.0, 8.0),
                scale: 8.0 * Vec3::ONE,
                rotation: Quat::default(),
            },
        )
        .unwrap();
    //let building = app.add_object("assets/city_building.glb").unwrap();

    scene.skybox = Some(
        SkyBox::load(
            &app.instance,
            &app.device,
            &mut data,
            "assets/skyboxes/nebula/top.png",
            "assets/skyboxes/nebula/bottom.png",
            "assets/skyboxes/nebula/left.png",
            "assets/skyboxes/nebula/right.png",
            "assets/skyboxes/nebula/back.png",
            "assets/skyboxes/nebula/front.png",
        )
        .unwrap(),
    );
}
