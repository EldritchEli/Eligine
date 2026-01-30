#![allow(unsafe_op_in_unsafe_fn, clippy::missing_safety_doc)]
use std::time::Instant;

use crate::{
    bevy_app::render::{destroy, render},
    game_objects::scene::Scene,
    gui::gui::{Gui, create_gui_from_window},
    vulkan::{
        color_objects::create_color_objects,
        command_buffer_util::create_command_buffers,
        command_pool::{create_command_pools, create_transient_command_pool},
        descriptor_util::{
            create_descriptor_pool, create_global_buffers, gui_descriptor_set_layout,
            pbr_descriptor_set_layout, skybox_descriptor_set_layout,
        },
        device_util::{create_logical_device, pick_physical_device},
        framebuffer_util::{create_depth_objects, create_framebuffers},
        input_state::InputState,
        instance_util::create_instance,
        pipeline_util::{create_pbr_pipeline, gui_pipeline, skybox_pipeline},
        render_app::{self, AppData, FrameInfo},
        render_pass_util::create_render_pass,
        swapchain_util::{create_swapchain, create_swapchain_image_views},
        sync_util::create_sync_objects,
    },
};
use bevy::{
    app::{App, PanicHandlerPlugin, Plugin, ScheduleRunnerPlugin, Startup, TaskPoolPlugin, Update},
    asset::AssetPlugin,
    diagnostic::DiagnosticsPlugin,
    ecs::{
        entity::Entity,
        message::{MessageReader, MessageWriter},
        query::With,
        resource::Resource,
        system::{Commands, NonSendMut, Query, Res, ResMut},
    },
    image::ImagePlugin,
    input::InputPlugin,
    time::{Time, TimePlugin},
    transform::TransformPlugin,
    window::{PrimaryWindow, WindowPlugin},
    winit::{RawWinitWindowEvent, WINIT_WINDOWS, WinitPlugin},
};

use vulkanalia::{Device, vk::DeviceV1_0, window as vk_window};
use vulkanalia::{
    Entry, Instance,
    loader::{LIBRARY, LibloadingLoader},
};
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};
#[derive(Resource, Clone, Debug)]

pub struct VkDevice {
    pub inner: Device,
}
#[derive(Resource, Clone, Debug)]
pub struct VkInstance {
    pub inner: Instance,
}
#[derive(Resource)]
pub struct AssetList(Vec<String>);
impl AssetList {
    pub fn new(list: Vec<String>) -> AssetList {
        AssetList(list)
    }
    pub fn inner_ref(&self) -> &Vec<String> {
        &self.0
    }
}

pub struct VulkanDefault;

impl Plugin for VulkanDefault {
    fn build(&self, app: &mut App) {
        app.add_plugins(PanicHandlerPlugin);
        app.add_plugins(TaskPoolPlugin::default());
        //app.add_plugins(FrameCountPlugin);
        app.add_plugins(TimePlugin);
        app.add_plugins(TransformPlugin);
        app.add_plugins(DiagnosticsPlugin);
        app.add_plugins(InputPlugin);
        app.add_plugins(AssetPlugin::default());
        app.add_plugins(ImagePlugin::default());
        app.add_plugins(bevy::a11y::AccessibilityPlugin);
        app.add_plugins(ScheduleRunnerPlugin::default());
        app.add_plugins(WinitPlugin::default());
        app.add_plugins(WindowPlugin::default());
        app.insert_resource(InputState::default());
        app.add_systems(Startup, (create_render_app, create_gui_from_window));
        app.add_systems(Update, (process_raw_winit_events, old_process_window_event));
    }
}

pub fn create_vulkan_resources(
    mut commands: Commands,
    primary_window: Query<Entity, With<PrimaryWindow>>,
) {
    WINIT_WINDOWS.with_borrow(|windows| {
        if let Some(window) = windows.get_window(primary_window.single().unwrap()) {
            let entity = primary_window.single().unwrap();
            let window = windows.get_window(entity).unwrap();
            unsafe {
                let resized = false;
                let loader = LibloadingLoader::new(LIBRARY).unwrap();
                let mut scene = Scene::default();
                let entry = Entry::new(loader)
                    .inspect_err(|_| eprintln!("failed to create entry"))
                    .unwrap();
                let mut data = AppData::default();
                let instance = create_instance(window, &entry, &mut data).unwrap();
                data.surface = vk_window::create_surface(
                    &instance,
                    &window.display_handle().unwrap(),
                    &window.window_handle().unwrap(),
                )
                .unwrap();
                pick_physical_device(&instance, &mut data).unwrap();
                let device = create_logical_device(&entry, &instance, &mut data).unwrap();
                let start = Instant::now();
                create_swapchain(window, &instance, &device, &mut data).unwrap();
                create_swapchain_image_views(&device, &mut data).unwrap();
                create_render_pass(&instance, &device, &mut data).unwrap();
                pbr_descriptor_set_layout(&device, &mut data).unwrap();
                skybox_descriptor_set_layout(&device, &mut data).unwrap();
                gui_descriptor_set_layout(&device, &mut data).unwrap();
                gui_pipeline(&device, &mut data, 0).unwrap();
                create_pbr_pipeline(&device, &mut data, 1).unwrap();
                skybox_pipeline(&device, &mut data, 2).unwrap();
                create_color_objects(&instance, &device, &mut data).unwrap();
                create_depth_objects(&instance, &device, &mut data).unwrap();
                create_framebuffers(&device, &mut data).unwrap();
                create_command_pools(&instance, &device, &mut data).unwrap();
                create_transient_command_pool(&instance, &device, &mut data).unwrap();
                create_descriptor_pool(&device, &mut data, 30).unwrap();

                create_command_buffers(&device, &mut scene, &mut data, window, None).unwrap();
                create_sync_objects(&device, &mut data).unwrap();

                create_global_buffers(&instance, &device, &mut data, &mut scene).unwrap();

                commands.insert_resource(VkDevice { inner: device });
                commands.insert_resource(VkInstance { inner: instance });
                commands.insert_resource(scene);
                commands.insert_resource(data);
                commands.insert_resource(FrameInfo {
                    resized,
                    frame: 0,
                    start,
                    time_stamp: 0.0,
                });
            }
        }
    });
}

fn init_gui(
    mut gui: NonSendMut<Gui>,
    instance: Res<VkInstance>,
    device: Res<VkDevice>,
    mut data: ResMut<render_app::AppData>,
    mut scene: ResMut<Scene>,

    window_query: Query<Entity, With<PrimaryWindow>>,
) {
    let w_id = window_query.single().unwrap();
    WINIT_WINDOWS.with_borrow(|windows| {
        if let Some(window) = windows.get_window(w_id) {
            let ppp = gui.egui_state.egui_ctx().pixels_per_point();
            let output = gui.run_egui_fst(&mut data, &mut scene, &window);
            /*gui.update_gui_images(
                &instance.inner,
                &device.inner,
                &mut data,
                &output.textures_delta,
            )
            .unwrap();
            println!("ther");
            gui.init_gui_mesh(&instance.inner, &device.inner, &mut data, &output, ppp)
                .unwrap();*/
        }
    });
}

fn create_render_app(
    mut commands: Commands,
    window_query: Query<Entity, With<PrimaryWindow>>,
    writer: MessageWriter<bevy::window::WindowEvent>,
) {
    // let raw_window = windows.get_window(entity).unwrap();
    let w_id = window_query.single().unwrap();
    WINIT_WINDOWS.with_borrow(|windows| {
        if let Some(window) = windows.get_window(w_id) {
            commands.insert_resource(unsafe { render_app::App::create(window).unwrap() });
            window.request_redraw();
        } else {
            println!("failed to find window")
        }
    });

    // Here you can use raw_window.canvas() or raw_window.raw_window_handle()
    // to initialize your custom WGPU Device, Vulkan Instance, etc.
}
pub fn process_raw_winit_events(
    mut input_state: ResMut<InputState>,
    mut gui: NonSendMut<Gui>,
    window_query: Query<Entity, With<PrimaryWindow>>,
    mut app: ResMut<render_app::App>,
    mut event_reader: MessageReader<RawWinitWindowEvent>,
    time: Res<Time>,
) {
    let w_id = window_query.single().unwrap();

    WINIT_WINDOWS.with_borrow(|windows| {
        if let Some(window) = windows.get_window(w_id) {
            for event in event_reader.read() {
                let response = gui.egui_state.on_window_event(window, &event.event);
                input_state.read_event(&event.event);
                gui.set_enabled(&mut input_state);
                app.scene.update(time.delta_secs(), &input_state);
            }
        }
    });
}

pub fn old_process_window_event(
    gui: NonSendMut<Gui>,
    mut app: ResMut<render_app::App>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
    mut event_reader: MessageReader<RawWinitWindowEvent>,
) {
    let mut gui = gui;
    let Some(event) = event_reader.read().next() else {
        println!("no event");
        return;
    };
    match event.event {
        winit::event::WindowEvent::Resized(size) => {
            if size.width == 0 || size.height == 0 {
            } else {
                app.resized = true;
            }
        }

        winit::event::WindowEvent::Destroyed => {
            println!("window destroyed");
            unsafe {
                app.device.device_wait_idle().unwrap();
            }
            unsafe {
                app.destroy(&mut gui);
            }
        }
        // Destroy our Vulkan app.
        winit::event::WindowEvent::CloseRequested => {
            println!("window closed");

            unsafe {
                app.destroy(&mut gui);
            }
        }
        winit::event::WindowEvent::RedrawRequested => {
            let entity = primary_window.single().unwrap();
            WINIT_WINDOWS.with_borrow(|windows| {
                let window = windows.get_window(entity).unwrap();

                let output = if gui.enabled {
                    Some(gui.old_run_egui_bevy(&mut app, window))
                } else {
                    None
                };
                /*if !output.textures_delta.is_empty() {
                    gui.new_texture_delta.push(output.textures_delta.clone())
                }*/
                unsafe { app.render(window, &mut gui, output) }.unwrap();
            })
        }
        _ =>
            /*WindowEvent::RedrawRequested if !event_loop.exiting() && !self.window_minimized => */
            {}
    }
}
pub fn process_window_event(
    gui: NonSendMut<Gui>,
    mut data: ResMut<AppData>,
    instance: Res<VkInstance>,
    mut device: Res<VkDevice>,
    mut frame_info: ResMut<FrameInfo>,
    mut scene: ResMut<Scene>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
    mut event_reader: MessageReader<RawWinitWindowEvent>,
) {
    let mut gui = gui;
    let Some(event) = event_reader.read().next() else {
        println!("no event");
        return;
    };
    dbg!(&instance);
    println!("got event");
    match event.event {
        winit::event::WindowEvent::Resized(size) => {
            if size.width == 0 || size.height == 0 {
            } else {
                frame_info.resized = true;
            }
        }

        winit::event::WindowEvent::Destroyed => {
            println!("window destroyed");
            unsafe {
                device.inner.device_wait_idle().unwrap();
            }
            unsafe {
                destroy(
                    &instance.inner,
                    &device.inner,
                    &mut data,
                    &mut scene,
                    &mut gui,
                );
            }
        }
        // Destroy our Vulkan app.
        winit::event::WindowEvent::CloseRequested => {
            println!("window closed");

            unsafe {
                destroy(
                    &instance.inner,
                    &device.inner,
                    &mut data,
                    &mut scene,
                    &mut gui,
                );
            }
        }
        winit::event::WindowEvent::RedrawRequested => {
            let entity = primary_window.single().unwrap();
            WINIT_WINDOWS.with_borrow(|windows| {
                let window = windows.get_window(entity).unwrap();

                let output = if gui.enabled {
                    Some(gui.run_egui(&mut data, &mut scene, window))
                } else {
                    None
                };
                /*if !output.textures_delta.is_empty() {
                    gui.new_texture_delta.push(output.textures_delta.clone())
                }*/
                render(
                    &instance.inner,
                    &device.inner,
                    &mut data,
                    &mut scene,
                    &mut frame_info,
                    window,
                    &mut gui,
                    output,
                );
            })
        }
        _ =>
            /*WindowEvent::RedrawRequested if !event_loop.exiting() && !self.window_minimized => */
            {}
    }
}
/*
    PanicHandlerPlugin
    LogPlugin - with feature bevy_log
    TaskPoolPlugin
    FrameCountPlugin
    TimePlugin
    TransformPlugin
    DiagnosticsPlugin
    InputPlugin
    ScheduleRunnerPlugin
    WindowPlugin - with feature bevy_window
    AccessibilityPlugin - with feature bevy_window
    TerminalCtrlCHandlerPlugin - with feature std
    AssetPlugin - with feature bevy_asset
    ScenePlugin - with feature bevy_scene
    WinitPlugin - with feature bevy_winit
    RenderPlugin - with feature bevy_render
    ImagePlugin - with feature bevy_render
    PipelinedRenderingPlugin - with feature bevy_render
    CorePipelinePlugin - with feature bevy_core_pipeline
    SpritePlugin - with feature bevy_sprite
    TextPlugin - with feature bevy_text
    UiPlugin - with feature bevy_ui
    PbrPlugin - with feature bevy_pbr
    GltfPlugin - with feature bevy_gltf
    AudioPlugin - with feature bevy_audio
    GilrsPlugin - with feature bevy_gilrs
    AnimationPlugin - with feature bevy_animation
    GizmoPlugin - with feature bevy_gizmos
    StatesPlugin - with feature bevy_state
    DevToolsPlugin - with feature bevy_dev_tools
    CiTestingPlugin - with feature bevy_ci_testing
    DefaultPickingPlugins - with feature bevy_picking

DefaultPlugins obeys Cargo feature flags. Users may exert control over this plugin group by disabling default-features in their Cargo.toml and enabling only those features that they wish to use.

DefaultPlugins contains all the plugins typically required to build a Bevy application which includes a window and presentation components. For the absolute minimum number of plugins needed to run a Bevy application, see MinimalPlugins. */
