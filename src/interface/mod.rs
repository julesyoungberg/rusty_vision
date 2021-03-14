use nannou::prelude::*;
use nannou::ui::prelude::*;
use nannou::ui::DrawToFrameError;

use crate::app;

mod audio_features_controls;
mod audio_fft_controls;
mod camera_info;
mod color_controls;
mod components;
mod errors;
mod geometry_controls;
mod image_controls;
mod noise_controls;

fn controls_height(model: &mut app::Model) -> f32 {
    let mut height = 140.0;

    let subscriptions = match &model.program_store.current_subscriptions {
        Some(s) => s,
        None => return height,
    };

    [
        subscriptions.audio_features,
        subscriptions.audio_fft,
        subscriptions.color,
        subscriptions.geometry,
        subscriptions.image,
        subscriptions.noise,
    ]
    .iter()
    .for_each(|s| {
        if *s {
            height += 60.0;
        }
    });

    height = height
        + audio_features_controls::height(model)
        + audio_fft_controls::height(model)
        + color_controls::height(model)
        + geometry_controls::height(model)
        + image_controls::height(model)
        + noise_controls::height(model);

    height
}

/// Main UI logic / layout
pub fn update(app: &App, device: &wgpu::Device, model: &mut app::Model) {
    let mut height = controls_height(model);
    let border = 40.0;
    let scroll = height > model.size[1] - border;
    if scroll {
        height = model.size[1] - border;
    }

    // Calling `set_widgets` allows us to instantiate some widgets.
    let ui = &mut model.ui.set_widgets();

    /////////////////////////
    // controls wrapper
    let mut controls_wrapper =
        components::container([220.0, height as f64]).top_left_with_margin(10.0);
    if scroll {
        controls_wrapper = controls_wrapper.scroll_kids_vertically();
    }
    controls_wrapper.set(model.widget_ids.controls_wrapper, ui);

    /////////////////////////
    // hint
    components::text_small(&"Press 'h' to hide".to_string())
        .parent(model.widget_ids.controls_wrapper)
        .top_left_with_margin(10.0)
        .set(model.widget_ids.toggle_controls_hint, ui);

    /////////////////////////
    // current folder select
    if let Some(folder_names) = &model.program_store.folder_names {
        components::label("Folder")
            .parent(model.widget_ids.controls_wrapper)
            .set(model.widget_ids.current_folder_label, ui);
        let names = folder_names
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<&str>>();
        if let Some(selected) = components::drop_down(&names[..], model.program_store.folder_index)
            .parent(model.widget_ids.controls_wrapper)
            .down(5.0)
            .set(model.widget_ids.current_folder, ui)
        {
            model.program_store.select_folder(app, device, selected);
        }
    }

    /////////////////////////
    // current program select
    if let Some(program_names) = &model.program_store.program_names {
        components::label("Shader")
            .parent(model.widget_ids.controls_wrapper)
            .set(model.widget_ids.current_program_label, ui);
        let names = program_names
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<&str>>();
        if let Some(selected) = components::drop_down(&names[..], model.program_store.program_index)
            .parent(model.widget_ids.controls_wrapper)
            .down(5.0)
            .set(model.widget_ids.current_program, ui)
        {
            model
                .program_store
                .select_program(app, device, selected, false);
        }
    }

    //////////////////////////////////////////////////
    // Error Display
    //////////////////////////////////////////////////
    let compile_errors = model.program_store.errors();
    if let Some(config_error) = &model.program_store.error {
        errors::update(
            &model.widget_ids,
            ui,
            "Config Error",
            config_error.as_str(),
            model.size,
        );
    } else if !compile_errors.is_none() && compile_errors.unwrap().keys().len() > 0 {
        errors::compilation_errors(&model.widget_ids, ui, &compile_errors.unwrap(), model.size);
    } else if let Some(audio_error) = &model.program_store.buffer_store.audio_source.error {
        errors::update(
            &model.widget_ids,
            ui,
            "Audio Error",
            audio_error.as_str(),
            model.size,
        );
    } else if let Some(audio_error) = &model
        .program_store
        .buffer_store
        .audio_features_uniforms
        .error
    {
        errors::update(
            &model.widget_ids,
            ui,
            "Audio Features Error",
            audio_error.as_str(),
            model.size,
        );
    }

    let subscriptions = match &model.program_store.current_subscriptions {
        Some(s) => s,
        None => return,
    };

    let mut left = -200.0;

    //////////////////////////////////////////////////
    // Color Controls
    //////////////////////////////////////////////////
    if subscriptions.color {
        for _click in components::button_big()
            .parent(model.widget_ids.controls_wrapper)
            .down(20.0)
            .label("Color")
            .set(model.widget_ids.general_folder, ui)
        {
            println!("toggle general controls");
            model.ui_show_color = !model.ui_show_color;
        }

        if model.ui_show_color {
            color_controls::update(
                &model.widget_ids,
                ui,
                &mut model.program_store.buffer_store.color_uniforms,
            );
            left = -60.0;
        }
    }

    //////////////////////////////////////////////////
    // Geometry Controls
    //////////////////////////////////////////////////
    if subscriptions.geometry {
        for _click in components::button_big()
            .parent(model.widget_ids.controls_wrapper)
            .down(20.0)
            .left(left as f64)
            .label("Geometry")
            .set(model.widget_ids.geometry_folder, ui)
        {
            println!("toggle geometry controls");
            model.ui_show_geometry = !model.ui_show_geometry;
        }

        left = 0.0;

        if model.ui_show_geometry {
            geometry_controls::update(
                &model.widget_ids,
                ui,
                &mut model.program_store.buffer_store.geometry_uniforms,
            );
        }
    }

    //////////////////////////////////////////////////
    // Audio Features Controls
    //////////////////////////////////////////////////
    if subscriptions.audio_features {
        for _click in components::button_big()
            .parent(model.widget_ids.controls_wrapper)
            .down(20.0)
            .left(left as f64)
            .label("Audio Features")
            .set(model.widget_ids.audio_features_folder, ui)
        {
            println!("toggle audio features controls");
            model.ui_show_audio_features = !model.ui_show_audio_features;
        }

        if model.ui_show_audio_features {
            audio_features_controls::update(
                &model.widget_ids,
                ui,
                &mut model.program_store.buffer_store.audio_features_uniforms,
            );
        }
    }

    //////////////////////////////////////////////////
    // Audio FFT Controls
    //////////////////////////////////////////////////
    if subscriptions.audio_fft {
        for _click in components::button_big()
            .parent(model.widget_ids.controls_wrapper)
            .down(20.0)
            .label("Audio FFT")
            .set(model.widget_ids.audio_fft_folder, ui)
        {
            println!("toggle audio fft controls");
            model.ui_show_audio_fft = !model.ui_show_audio_fft;
        }

        if model.ui_show_audio_fft {
            audio_fft_controls::update(
                &model.widget_ids,
                ui,
                &mut model.program_store.buffer_store.audio_fft_uniforms,
            );
        }
    }

    //////////////////////////////////////////////////
    // Noise Controls
    //////////////////////////////////////////////////
    if subscriptions.noise {
        for _click in components::button_big()
            .parent(model.widget_ids.controls_wrapper)
            .down(20.0)
            .label("Noise")
            .set(model.widget_ids.noise_folder, ui)
        {
            println!("toggle noise controls");
            model.ui_show_noise = !model.ui_show_noise;
        }

        if model.ui_show_noise {
            noise_controls::update(
                &model.widget_ids,
                ui,
                &mut model.program_store.buffer_store.noise_uniforms,
            );
        }
    }

    //////////////////////////////////////////////////
    // Image Controls
    //////////////////////////////////////////////////
    if subscriptions.image {
        for _click in components::button_big()
            .parent(model.widget_ids.controls_wrapper)
            .down(20.0)
            .label("Image")
            .set(model.widget_ids.image_folder, ui)
        {
            println!("toggle image controls");
            model.ui_show_image = !model.ui_show_image;
        }

        if model.ui_show_image {
            image_controls::update(
                app,
                &model.widget_ids,
                ui,
                &mut model.program_store.buffer_store.image_uniforms,
            );
        }
    }

    //////////////////////////////////////////////////
    // Other UI
    //////////////////////////////////////////////////
    if subscriptions.camera {
        camera_info::update(
            &model.widget_ids,
            ui,
            &mut model.program_store.buffer_store.camera_uniforms,
        );
    }

    components::container([80.0, 35.0])
        .no_parent()
        .bottom_right_with_margin(10.0)
        .set(model.widget_ids.fps_container, ui);

    components::text(&format!("FPS: {:.2}", app.fps()))
        .parent(model.widget_ids.fps_container)
        .top_left_with_margin(10.0)
        .set(model.widget_ids.fps, ui);
}

/// Draw the state of the `Ui` to the frame.
pub fn draw(app: &App, model: &app::Model, frame: &Frame) {
    let color_attachment_desc = frame.color_attachment_descriptor();
    let primitives = model.ui.draw();
    let window = app
        .window(model.main_window_id)
        .ok_or(DrawToFrameError::InvalidWindow)
        .unwrap();
    let mut ui_encoder = frame.command_encoder();
    ui::encode_render_pass(
        &model.ui,
        &window,
        primitives,
        color_attachment_desc,
        &mut *ui_encoder,
    )
    .unwrap();
}
