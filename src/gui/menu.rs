use egui::Context;

pub fn show_menu(ctx: &Context) {
    egui::TopBottomPanel::top("my_panel").show(ctx, |ui| {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Quit").clicked() {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });
        });
    });
}
