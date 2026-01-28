use bevy::{
    ecs::{
        entity::Entity,
        message::MessageReader,
        query::With,
        system::{NonSend, Query, ResMut},
    },
    window::{PrimaryWindow, WindowResized},
    winit::WinitWindows,
};

fn manage_renderer_lifecycle(
    windows: NonSend<WinitWindows>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
    mut resize_events: MessageReader<WindowResized>,
) {
    let Ok(window_entity) = primary_window.single() else {
        return;
    };
    let Some(winit_window) = windows.get_window(window_entity) else {
        return;
    };

    // 1. Handle Surface Creation (Initialization or Resumption)
    /*if renderer.surface.is_none() {
        let surface = renderer.instance.create_surface(winit_window).unwrap();
        // Configure surface here (size, format, etc.)
        renderer.surface = Some(surface);
    }*/

    // 2. Handle Resizing
    for _ in resize_events.read() {
        let size = winit_window.inner_size();
        // reconfigure_surface(&renderer.device, renderer.surface.as_ref().unwrap(), size);
    }
}
