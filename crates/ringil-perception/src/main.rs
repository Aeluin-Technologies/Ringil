use std::sync::Arc;

use dora_node_api::arrow::array::{
    Array, Float32Array, ListArray, StructArray, UInt8Array, UInt32Array,
    UInt64Array,
};
use dora_node_api::arrow::datatypes::{DataType, Field};
use dora_node_api::{self, DoraNode, Event, dora_core::config::DataId};
use image::{DynamicImage, RgbImage};

use ringil_perception::events::InstinctEvent;
use ringil_perception::pipeline::InstinctPipeline;

fn main() -> eyre::Result<()> {
    let (mut node, mut events) = DoraNode::init_from_env()?;
    let mut pipeline = InstinctPipeline::new()?;

    let output_obstacles = DataId::from("obstacles");

    tracing::info!("ringil perception node started");

    while let Some(event) = events.recv() {
        match event {
            Event::Input { id, metadata, data } => {
                if id.as_str() == "image" {
                    let array: Arc<dyn Array> = data.into();

                    let image = match decode_ros2_image(&array) {
                        Ok(img) => img,
                        Err(err) => {
                            tracing::error!(
                                ?err,
                                "failed to decode image from ros2"
                            );
                            continue;
                        },
                    };

                    if let Ok(instinct_events) = pipeline.process_frame(image)
                        && let Some(arrow_obstacles) =
                            encode_obstacles_to_arrow(&instinct_events)
                    {
                        node.send_output(
                            output_obstacles.clone(),
                            metadata.parameters,
                            arrow_obstacles,
                        )?;
                    }
                }
            },
            Event::Stop(_) => {
                tracing::info!("received stop signal, shutting down");
                break;
            },
            _ => {},
        }

        while let Ok(buffalo_event) = pipeline.buffalo_rx.try_recv() {
            if let InstinctEvent::PersonIdentityExtracted {
                track_id,
                embedding: _,
            } = buffalo_event
            {
                // TODO: encode the embedding (Vec<f32>) to Arrow and send it
                // to ringil-communication for swarm Re-ID sharing.
                tracing::debug!(?track_id, "extracted buffalo reid for track");
            }
        }
    }

    Ok(())
}

/// Decodes a ROS 2 `sensor_msgs/Image` into an `image::DynamicImage`
fn decode_ros2_image(data: &Arc<dyn Array>) -> eyre::Result<DynamicImage> {
    let struct_array = data
        .as_any()
        .downcast_ref::<StructArray>()
        .ok_or_else(|| eyre::eyre!("Expected StructArray for ROS 2 Image"))?;

    let width_arr = struct_array
        .column_by_name("width")
        .ok_or_else(|| {
            eyre::eyre!("Missing 'width' column in ROS 2 image struct")
        })?
        .as_any()
        .downcast_ref::<UInt32Array>()
        .ok_or_else(|| eyre::eyre!("'width' column is not a UInt32Array"))?;

    let height_arr = struct_array
        .column_by_name("height")
        .ok_or_else(|| {
            eyre::eyre!("Missing 'height' column in ROS 2 image struct")
        })?
        .as_any()
        .downcast_ref::<UInt32Array>()
        .ok_or_else(|| eyre::eyre!("'height' column is not a UInt32Array"))?;

    let raw_data_list = struct_array
        .column_by_name("data")
        .ok_or_else(|| {
            eyre::eyre!("Missing 'data' column in ROS 2 image struct")
        })?
        .as_any()
        .downcast_ref::<ListArray>()
        .ok_or_else(|| eyre::eyre!("'data' column is not a ListArray"))?;

    let raw_array = raw_data_list
        .value(0)
        .as_any()
        .downcast_ref::<UInt8Array>()
        .ok_or_else(|| eyre::eyre!("List data element is not a UInt8Array"))?
        .clone();

    let width = width_arr.value(0);
    let height = height_arr.value(0);
    let pixel_data: Vec<u8> = raw_array.values().to_vec();

    let img =
        RgbImage::from_raw(width, height, pixel_data).ok_or_else(|| {
            eyre::eyre!("Failed to construct RgbImage from Arrow byte buffer")
        })?;

    Ok(DynamicImage::ImageRgb8(img))
}

/// Encodes a list of `ObstacleDetected` events into an Apache Arrow `StructArray`
fn encode_obstacles_to_arrow(events: &[InstinctEvent]) -> Option<StructArray> {
    let mut ids = Vec::new();
    let mut classes = Vec::new();
    let mut cxs = Vec::new();
    let mut cys = Vec::new();
    let mut widths = Vec::new();
    let mut heights = Vec::new();
    let mut confs = Vec::new();

    for event in events {
        if let InstinctEvent::ObstacleDetected {
            id,
            class,
            obb,
            confidence,
        } = event
        {
            ids.push(*id);
            classes.push(*class as u8);
            cxs.push(obb.cx);
            cys.push(obb.cy);
            widths.push(obb.width);
            heights.push(obb.height);
            confs.push(*confidence);
        }
    }

    if ids.is_empty() {
        return None;
    }

    let fields = vec![
        Arc::new(Field::new("id", DataType::UInt64, false)),
        Arc::new(Field::new("class", DataType::UInt8, false)),
        Arc::new(Field::new("cx", DataType::Float32, false)),
        Arc::new(Field::new("cy", DataType::Float32, false)),
        Arc::new(Field::new("width", DataType::Float32, false)),
        Arc::new(Field::new("height", DataType::Float32, false)),
        Arc::new(Field::new("confidence", DataType::Float32, false)),
    ];

    let arrays: Vec<Arc<dyn Array>> = vec![
        Arc::new(UInt64Array::from(ids)),
        Arc::new(UInt8Array::from(classes)),
        Arc::new(Float32Array::from(cxs)),
        Arc::new(Float32Array::from(cys)),
        Arc::new(Float32Array::from(widths)),
        Arc::new(Float32Array::from(heights)),
        Arc::new(Float32Array::from(confs)),
    ];

    StructArray::try_new(fields.into(), arrays, None)
        .map_err(|err| {
            tracing::error!(?err, "failed to construct arrow StructArray")
        })
        .ok()
}
