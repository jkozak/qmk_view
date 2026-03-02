use eframe::NativeOptions;
use egui::ViewportBuilder;
use qmkview_core::config::WindowConfig;

pub fn create_overlay_options(config: &WindowConfig) -> NativeOptions {
    let mut builder = ViewportBuilder::default()
        .with_decorations(false)
        .with_transparent(true)
        .with_inner_size([config.width, config.height])
        .with_position([config.x, config.y])
        .with_resizable(false);

    if config.always_on_top {
        builder = builder.with_always_on_top();
    }

    if config.click_through {
        builder = builder.with_mouse_passthrough(true);
    }

    NativeOptions {
        viewport: builder,
        ..Default::default()
    }
}
