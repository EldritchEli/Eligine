use crate::{
    gltf,
    gui::gui::{Gui, create_gui_from_window},
};
use bevy::{
    app::{PanicHandlerPlugin, ScheduleRunnerPlugin},
    diagnostic::DiagnosticsPlugin,
    input::InputPlugin,
    prelude::*,
    scene::ScenePlugin,
    time::TimePlugin,
    window::PrimaryWindow,
    winit::{WinitPlugin, WinitWindows},
};
use log::info;
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

pub fn load_assets(mut _commands: Commands, _asset_server: Res<AssetServer>, list: Res<AssetList>) {
    for asset_path in list.inner_ref().clone() {
        info!("loading path: {}", asset_path);
        /*if let Err(e) = gltf::load::scene(asset_path) {
            error!(error = ?e);
        }*/
    }
}

pub fn print_assets(to_render: Query<&Mesh3d>) {
    for mesh in to_render {
        println!("mesh_id: {}", mesh.id());
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
        app.add_plugins(ScheduleRunnerPlugin::default());
        app.add_plugins(AssetPlugin::default());
        app.add_plugins(ScenePlugin);
        // app.init_asset::<bevy_pbr::prelude::StandardMaterial>();

        app.add_plugins(WindowPlugin::default());
        app.add_plugins(WinitPlugin::default());
        app.add_systems(Startup, create_gui_from_window);
        //add
        //app.insert_non_send_resource(Gui);
    }
}

fn setup_custom_renderer(
    primary_window: Query<Entity, With<PrimaryWindow>>,
    windows: NonSend<WinitWindows>,
) {
    let entity = primary_window.single().unwrap();
    let raw_window = windows.get_window(entity).unwrap();

    // Here you can use raw_window.canvas() or raw_window.raw_window_handle()
    // to initialize your custom WGPU Device, Vulkan Instance, etc.
    println!(
        "Custom renderer initialized for window: {:?}",
        raw_window.id()
    );
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

pub fn create_app(asset_list: AssetList) -> App {
    let mut app = App::new();

    app.add_plugins(
        VulkanDefault, //.disable::<bevy::core_pipeline::CorePipelinePlugin>()
    );
    app.insert_resource(asset_list);
    app.add_systems(Startup, load_assets);
    app.add_systems(Update, print_assets);

    app
}
