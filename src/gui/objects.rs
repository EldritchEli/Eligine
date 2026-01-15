use bevy::math::{Quat, ops::round};
use egui::{Context, DragValue, RichText, ScrollArea, TextEdit, Ui};

use crate::{
    game_objects::{
        render_object::ObjectId,
        scene::{ObjectSlab, Scene},
    },
    gui::objects,
};

pub fn show_objects(scene: &mut Scene, ctx: &Context, ui: &mut Ui) {
    // selected_object(scene, ctx, ui);
    ScrollArea::vertical().show(ui, |ui| {
        for (i, object) in scene.objects.iter_mut() {
            if object.parent.is_none() && ui.button(object.name.clone()).clicked() {
                scene.selected_object = ObjectId(i);
            };
        }
    });
}

pub fn selected_object(scene: &mut Scene, ctx: &Context, ui: &mut Ui) {
    let Some(object) = scene.objects.get_mut(scene.selected_object) else {
        return;
    };
    let pos = &mut object.transform.position;
    let scale = &mut object.transform.scale;
    let mut rotation = object.transform.rotation.to_euler(glam::EulerRot::XYZ);

    ui.separator();
    ui.label(RichText::new(object.name.clone()).size(28.0));
    ui.label(RichText::new("position"));
    ui.horizontal(|ui| {
        ui.label("x");
        ui.add(DragValue::new(&mut pos.x).speed(0.1));
        ui.label("y");
        ui.add(DragValue::new(&mut pos.y).speed(1));
        ui.label("z");
        ui.add(DragValue::new(&mut pos.z).speed(1));
    });
    ui.label(RichText::new("scale"));
    ui.horizontal(|ui| {
        ui.label("x");
        ui.add(DragValue::new(&mut scale.x).speed(0.1));
        ui.label("y");
        ui.add(DragValue::new(&mut scale.y).speed(0.1));
        ui.label("z");
        ui.add(DragValue::new(&mut scale.z).speed(0.1));
    });
    ui.label(RichText::new("rotation"));
    ui.horizontal(|ui| {
        ui.label("x");
        if ui.add(DragValue::new(&mut rotation.0).speed(0.1)).changed() {
            object.transform.rotation =
                glam::Quat::from_euler(glam::EulerRot::XYZ, rotation.0, rotation.1, rotation.2);
        };
        ui.label("y");
        if ui.add(DragValue::new(&mut rotation.1).speed(0.1)).changed() {
            object.transform.rotation =
                glam::Quat::from_euler(glam::EulerRot::XYZ, rotation.0, rotation.1, rotation.2);
        };
        ui.label("z");
        if ui.add(DragValue::new(&mut rotation.2).speed(0.1)).changed() {
            object.transform.rotation =
                glam::Quat::from_euler(glam::EulerRot::XYZ, rotation.0, rotation.1, rotation.2);
        };
    });
}
