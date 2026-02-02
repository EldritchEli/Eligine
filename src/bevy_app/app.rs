#![allow(unsafe_op_in_unsafe_fn, clippy::missing_safety_doc)]

use crate::{
    bevy_app::render::{VulkanApp, create_vulkan_resources, destroy, render},
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
        render_pass_util::create_render_pass,
        swapchain_util::{create_swapchain, create_swapchain_image_views},
        sync_util::create_sync_objects,
    },
    winit_app::winit_render_app::{self, AppData},
};
use bevy::{
    app::{
        App, PanicHandlerPlugin, Plugin, PostUpdate, ScheduleRunnerPlugin, Startup, TaskPoolPlugin,
        Update,
    },
    asset::AssetPlugin,
    diagnostic::DiagnosticsPlugin,
    ecs::{
        entity::Entity,
        message::{Message, MessageReader, MessageWriter},
        query::With,
        resource::Resource,
        system::{Commands, NonSendMut, Query, Res, ResMut},
    },
    image::ImagePlugin,
    input::InputPlugin,
    time::{Time, TimePlugin},
    transform::TransformPlugin,
    window::{PrimaryWindow, RequestRedraw, WindowCloseRequested, WindowDestroyed, WindowPlugin},
    winit::{RawWinitWindowEvent, WINIT_WINDOWS, WinitPlugin},
};

use tracing::info;
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
        app.add_systems(Startup, (create_vulkan_resources, create_gui_from_window));
        app.add_systems(
            Update,
            (process_raw_winit_events, redraw /*destroy_renderer*/),
        );
        //app.add_systems(Update, update_camera_and_gui);
        app.add_systems(
            PostUpdate,
            (redraw, |mut writer: MessageWriter<RequestRedraw>| {
                writer.write(RequestRedraw);
            }),
        );
    }
}

pub fn create_render_app(
    mut commands: Commands,
    window_query: Query<Entity, With<PrimaryWindow>>,
    mut writer: MessageWriter<RequestRedraw>,
) {
    // let raw_window = windows.get_window(entity).unwrap();
    let w_id = window_query.single().unwrap();
    WINIT_WINDOWS.with_borrow(|windows| {
        if let Some(window) = windows.get_window(w_id) {
            commands.insert_resource(unsafe { winit_render_app::App::create(window).unwrap() });
            writer.write(RequestRedraw);
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
    mut app: ResMut<VulkanApp>,
    mut data: ResMut<AppData>,
    mut scene: ResMut<Scene>,
    mut event_reader: MessageReader<RawWinitWindowEvent>,
    time: Res<Time>,
) {
    let w_id = window_query.single().unwrap();

    WINIT_WINDOWS.with_borrow(|windows| {
        if let Some(window) = windows.get_window(w_id) {
            for event in event_reader.read() {
                match event.event {
                    winit::event::WindowEvent::Resized(size) => {
                        if size.width == 0 || size.height == 0 {
                        } else {
                            app.resized = true;
                        };
                    }
                    _ => {}
                }
                if gui.enabled {
                    let _response = gui.egui_state.on_window_event(window, &event.event);
                }
                input_state.read_event(&event.event);
            }
            gui.set_enabled(&mut input_state);
            scene.update(time.delta_secs(), &input_state);
            input_state.reset_mouse_delta();
        }
    });
}

pub fn destroy_renderer(
    gui: NonSendMut<Gui>,
    mut app: ResMut<VulkanApp>,
    mut scene: ResMut<Scene>,
    mut data: ResMut<AppData>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
    mut event_reader: MessageReader<WindowCloseRequested>,
) {
    let mut gui = gui;
    let Some(event) = event_reader.read().next() else {
        return;
    };
    // Destroy our Vulkan app.
    println!("window closed");

    unsafe {
        app.device.device_wait_idle().unwrap();
    }
    let app = &mut app;
    unsafe {
        destroy(app, &mut data, &mut scene, &mut gui);
    }
}

pub fn received_redraw(mut event_reader: MessageReader<RequestRedraw>) -> bool {
    event_reader.read().next().is_some()
}
pub fn redraw(
    gui: NonSendMut<Gui>,
    mut app: ResMut<VulkanApp>,
    mut scene: ResMut<Scene>,
    mut data: ResMut<AppData>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
    mut event_reader: MessageReader<RequestRedraw>,
) {
    let mut gui = gui;
    let Some(event) = event_reader.read().next() else {
        info!("no primary window found, probably shutting down");
        return;
    };
    let Ok(entity) = primary_window.single() else {
        return;
    };
    WINIT_WINDOWS.with_borrow(|windows| {
        let window = windows.get_window(entity).unwrap();

        gui.old_run_egui_bevy(&mut app, &mut data, &mut scene, window);
        unsafe {
            crate::bevy_app::render::render(&mut app, &mut data, &mut scene, window, &mut gui)
        };
    })
}
/*pub fn process_window_event(
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
        return;
    };
    dbg!(&instance);
    println!("got event");
    match event.event {
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
