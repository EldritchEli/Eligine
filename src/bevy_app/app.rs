#![allow(unsafe_op_in_unsafe_fn, clippy::missing_safety_doc)]

use crate::{
    bevy_app::render::{self, VulkanApp, create_vulkan_resources, destroy},
    game_objects::scene::Scene,
    gui::gui::{Gui, create_gui_from_window},
    vulkan::input_state::InputState,
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
        message::{MessageReader, MessageWriter},
        query::With,
        resource::Resource,
        schedule::IntoScheduleConfigs,
        system::{Commands, NonSendMut, Query, Res, ResMut},
    },
    image::ImagePlugin,
    input::InputPlugin,
    time::{Time, TimePlugin},
    transform::TransformPlugin,
    window::{PrimaryWindow, RequestRedraw, WindowCloseRequested, WindowPlugin},
    winit::{RawWinitWindowEvent, WINIT_WINDOWS, WinitPlugin},
};

use vulkanalia::Instance;
use vulkanalia::{Device, vk::DeviceV1_0};
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
            (
                process_raw_winit_events,
                update_gui,
                render::handle_acquired_image,
                render::update_uniforms,
                render::submit_command_buffer_and_present_image,
            )
                .chain()
                .run_if(received_redraw),
        );
        //app.add_systems(Update, update_camera_and_gui);
        app.add_systems(PostUpdate, |mut writer: MessageWriter<RequestRedraw>| {
            writer.write(RequestRedraw);
        });
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
    mut scene: ResMut<Scene>,
    mut event_reader: MessageReader<RawWinitWindowEvent>,
    time: Res<Time>,
) {
    let w_id = window_query.single().unwrap();

    WINIT_WINDOWS.with_borrow(|windows| {
        if let Some(window) = windows.get_window(w_id) {
            for event in event_reader.read() {
                if let winit::event::WindowEvent::Resized(size) = event.event {
                    if size.width == 0 || size.height == 0 {
                    } else {
                        app.resized = true;
                    };
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
    mut event_reader: MessageReader<WindowCloseRequested>,
) {
    let mut gui = gui;
    let Some(_event) = event_reader.read().next() else {
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
pub fn update_gui(
    mut gui: NonSendMut<Gui>,
    mut app: ResMut<VulkanApp>,
    mut scene: ResMut<Scene>,
    mut data: ResMut<AppData>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
) {
    let mut gui = gui;
    let Ok(entity) = primary_window.single() else {
        return;
    };
    WINIT_WINDOWS.with_borrow(|windows| {
        let window = windows.get_window(entity).unwrap();
        gui.old_run_egui_bevy(&mut app, &mut data, &mut scene, window);
    })
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
