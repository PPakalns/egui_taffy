use crate::TaffyContainerUi;

pub trait InteractiveWidget {
    fn create(ui: &mut egui::Ui, container: &TaffyContainerUi) -> Option<egui::Response>;
}

pub struct InteractTransparent;

impl InteractiveWidget for InteractTransparent {
    fn create(ui: &mut egui::Ui, container: &TaffyContainerUi) -> Option<egui::Response> {
        let _ = container;
        let _ = ui;
        None
    }
}

pub struct InteractOpaque;

impl InteractiveWidget for InteractOpaque {
    fn create(ui: &mut egui::Ui, container: &TaffyContainerUi) -> Option<egui::Response> {
        let rect = container.full_container();
        let _ = ui.interact(rect, ui.id().with("bg"), egui::Sense::click_and_drag());
        // Drop interaction, just opaque background
        None
    }
}

pub struct Interact;

impl InteractiveWidget for Interact {
    fn create(ui: &mut egui::Ui, container: &TaffyContainerUi) -> Option<egui::Response> {
        let rect = container.full_container();
        Some(ui.interact(rect, ui.id().with("bg"), egui::Sense::click()))
    }
}
