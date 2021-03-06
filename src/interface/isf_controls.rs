use nannou::prelude::*;
use nannou::ui::prelude::*;

use crate::app;
use crate::interface::components;
use crate::programs::isf::data;
use crate::programs::isf::IsfPipeline;

pub fn height(model: &mut app::Model) -> f32 {
    let isf = match &model.program_store.isf_pipeline {
        Some(isf_pipeline) => match &isf_pipeline.isf {
            Some(isf) => isf,
            None => return 0.0,
        },
        None => return 0.0,
    };

    if isf.inputs.is_empty() {
        return 0.0;
    }

    let mut height = 30.0;

    for input in &isf.inputs {
        match &input.ty {
            isf::InputType::Float(_) | isf::InputType::Bool(_) | isf::InputType::Event { .. } => {
                height += 35.0;
            }
            isf::InputType::Long(_)
            | isf::InputType::Point2d(_)
            | isf::InputType::Color(_)
            | isf::InputType::Image { .. } => {
                height += 55.0;
            }
            _ => (),
        };
    }

    height
}

pub fn update(
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    widget_ids: &app::WidgetIds,
    ui: &mut UiCell,
    isf_pipeline: &mut IsfPipeline,
    size: Point2,
) {
    let isf = match &isf_pipeline.isf {
        Some(isf) => isf,
        None => return,
    };

    if isf.inputs.is_empty() {
        return;
    }

    if let Some(isf_widget_ids) = &isf_pipeline.widget_ids.as_ref() {
        widget::Text::new("ISF Inputs")
            .rgb(0.9, 0.9, 0.9)
            .font_size(18)
            .parent(widget_ids.controls_wrapper)
            .down(10.0)
            .set(widget_ids.isf_inputs_title, ui);

        let data_inputs = isf_pipeline.isf_data.inputs_mut();

        for input in &isf.inputs {
            let data = data_inputs.get_mut(&input.name).unwrap();

            match (data, &input.ty) {
                (data::IsfInputData::Bool(val), isf::InputType::Bool(_)) => {
                    let widget_id = match isf_widget_ids.get(&input.name) {
                        Some(id) => id,
                        None => continue,
                    };

                    for _click in components::button_small_wide(*val)
                        .parent(widget_ids.controls_wrapper)
                        .down(10.0)
                        .align_left_of(widget_ids.controls_wrapper)
                        .label(&input.name)
                        .set(*widget_id, ui)
                    {
                        *val = !*val;
                    }
                }
                (data::IsfInputData::Event { happening }, isf::InputType::Event) => {
                    let widget_id = match isf_widget_ids.get(&input.name) {
                        Some(id) => id,
                        None => continue,
                    };

                    for _click in components::button_small_wide(false)
                        .parent(widget_ids.controls_wrapper)
                        .down(10.0)
                        .align_left_of(widget_ids.controls_wrapper)
                        .label(&input.name)
                        .set(*widget_id, ui)
                    {
                        *happening = true;
                    }
                }
                (data::IsfInputData::Float(val), isf::InputType::Float(input_config)) => {
                    let widget_id = match isf_widget_ids.get(&input.name) {
                        Some(id) => id,
                        None => continue,
                    };

                    if let Some(value) = components::slider(
                        *val,
                        input_config.min.unwrap_or(0.0),
                        input_config.max.unwrap_or(1.0),
                    )
                    .parent(widget_ids.controls_wrapper)
                    .down(10.0)
                    .align_left_of(widget_ids.controls_wrapper)
                    .label(input.name.as_str())
                    .set(*widget_id, ui)
                    {
                        *val = value;
                    }
                }
                (
                    data::IsfInputData::Long { selected, value },
                    isf::InputType::Long(input_config),
                ) => {
                    let widget_id = match isf_widget_ids.get(&input.name) {
                        Some(id) => id,
                        None => continue,
                    };

                    let mut label_name = input.name.clone();
                    label_name.push_str("-label");

                    components::label(input.name.as_str())
                        .parent(widget_ids.controls_wrapper)
                        .align_left_of(widget_ids.controls_wrapper)
                        .set(*isf_widget_ids.get(&label_name).unwrap(), ui);

                    let labels = input_config
                        .labels
                        .iter()
                        .map(|l| l.as_str())
                        .collect::<Vec<&str>>();

                    if let Some(index) = components::drop_down(&labels[..], *selected)
                        .parent(widget_ids.controls_wrapper)
                        .down(5.0)
                        .set(*widget_id, ui)
                    {
                        *selected = index;
                        *value = input_config.values[index];
                    }
                }
                (data::IsfInputData::Image(image_input), isf::InputType::Image) => {
                    let widget_id = match isf_widget_ids.get(&input.name) {
                        Some(id) => id,
                        None => continue,
                    };

                    let mut label_name = input.name.clone();
                    label_name.push_str("-label");

                    components::label(input.name.as_str())
                        .align_left_of(widget_ids.controls_wrapper)
                        .set(*isf_widget_ids.get(&label_name).unwrap(), ui);

                    let labels = &["image", "video", "webcam"];
                    let selected = match &image_input.source {
                        data::ImageSource::Image(_) => 0,
                        data::ImageSource::Video(_) => 1,
                        data::ImageSource::Webcam(_) => 2,
                        _ => 0,
                    };

                    if let Some(index) = components::drop_down(labels, selected)
                        .parent(widget_ids.controls_wrapper)
                        .down(5.0)
                        .set(*widget_id, ui)
                    {
                        match labels[index] {
                            "image" => {
                                image_input.select_image(
                                    device,
                                    encoder,
                                    &isf_pipeline.image_loader,
                                );
                            }
                            "video" => {
                                image_input.select_video(device);
                            }
                            "webcam" => {
                                image_input.start_webcam(device, size);
                            }
                            _ => {}
                        };
                        isf_pipeline.updated = true;
                    }
                }
                (data::IsfInputData::Point2d(val), isf::InputType::Point2d(input_config)) => {
                    let min = input_config.min.unwrap_or([0.0, 0.0]);
                    let max = input_config.max.unwrap_or([size[0] * 2.0, size[1] * 2.0]);

                    let mut label_name = input.name.clone();
                    label_name.push_str("-label");
                    let mut x_name = input.name.clone();
                    x_name.push_str("-x");
                    let mut y_name = input.name.clone();
                    y_name.push_str("-y");

                    components::label(input.name.as_str())
                        .align_left_of(widget_ids.controls_wrapper)
                        .parent(widget_ids.controls_wrapper)
                        .set(*isf_widget_ids.get(&label_name).unwrap(), ui);

                    if let Some(value) = components::x_2d_slider(val[0], min[0], max[0])
                        .parent(widget_ids.controls_wrapper)
                        .set(*isf_widget_ids.get(&x_name).unwrap(), ui)
                    {
                        val[0] = value;
                    }

                    if let Some(value) = components::y_2d_slider(val[1], min[1], max[1])
                        .parent(widget_ids.controls_wrapper)
                        .set(*isf_widget_ids.get(&y_name).unwrap(), ui)
                    {
                        val[1] = value;
                    }
                }
                (data::IsfInputData::Color(val), isf::InputType::Color(input_config)) => {
                    let min = input_config
                        .min
                        .clone()
                        .unwrap_or_else(|| vec![0.0, 0.0, 0.0, 0.0]);
                    let max = input_config
                        .max
                        .clone()
                        .unwrap_or_else(|| vec![1.0, 1.0, 1.0, 1.0]);

                    let mut label_name = input.name.clone();
                    label_name.push_str("-label");
                    let mut r_name = input.name.clone();
                    r_name.push_str("-r");
                    let mut g_name = input.name.clone();
                    g_name.push_str("-g");
                    let mut b_name = input.name.clone();
                    b_name.push_str("-b");
                    let mut a_name = input.name.clone();
                    a_name.push_str("-a");

                    components::label(input.name.as_str())
                        .align_left_of(widget_ids.controls_wrapper)
                        .parent(widget_ids.controls_wrapper)
                        .set(*isf_widget_ids.get(&label_name).unwrap(), ui);

                    if let Some(value) = components::r_4d_slider(val.red, min[0], max[0])
                        .parent(widget_ids.controls_wrapper)
                        .set(*isf_widget_ids.get(&r_name).unwrap(), ui)
                    {
                        val.red = value;
                    }

                    if let Some(value) = components::g_4d_slider(val.green, min[1], max[1])
                        .parent(widget_ids.controls_wrapper)
                        .set(*isf_widget_ids.get(&g_name).unwrap(), ui)
                    {
                        val.green = value;
                    }

                    if let Some(value) = components::b_4d_slider(val.blue, min[2], max[2])
                        .parent(widget_ids.controls_wrapper)
                        .set(*isf_widget_ids.get(&b_name).unwrap(), ui)
                    {
                        val.blue = value;
                    }

                    if let Some(value) = components::a_4d_slider(val.alpha, min[3], max[3])
                        .parent(widget_ids.controls_wrapper)
                        .set(*isf_widget_ids.get(&a_name).unwrap(), ui)
                    {
                        val.alpha = value;
                    }
                }
                _ => (),
            };
        }
    }
}
