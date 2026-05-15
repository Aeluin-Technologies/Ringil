use std::collections::HashMap;
use std::path::Path;

use ab_glyph::{FontArc, PxScale};
use eyre::Result;
use ffmpeg::format::{Pixel, input, output};
use ffmpeg::media::Type;
use ffmpeg::software::scaling::{
    context::Context as ScalerContext, flag::Flags,
};
use ffmpeg::util::frame::Video as VideoFrame;
use ffmpeg_next as ffmpeg;
use image::{DynamicImage, Rgb, RgbImage};
use imageproc::drawing::{draw_hollow_rect_mut, draw_text_mut};
use imageproc::rect::Rect;

use ringil_perception::{
    InstinctEvent, InstinctPipeline, ObjectClass, OrientedBoundingBox,
};

#[test]
#[ignore]
fn test_render_annotated_video() -> Result<()> {
    ffmpeg::init()?;
    let mut pipeline = InstinctPipeline::new()?;

    let manifest_dir = option_env!("CARGO_MANIFEST_DIR")
        .ok_or_else(|| eyre::eyre!("CARGO_MANIFEST_DIR missing"))?;
    let input_path = Path::new(manifest_dir)
        .join("tests")
        .join("test_sample.mp4");
    let output_path =
        Path::new(manifest_dir).join("../../output_annotated.mp4");

    let mut ictx = input(&input_path)?;
    let input_stream = ictx
        .streams()
        .best(Type::Video)
        .ok_or_else(|| eyre::eyre!("No video stream"))?;
    let video_stream_index = input_stream.index();

    let mut decoder = ffmpeg::codec::context::Context::from_parameters(
        input_stream.parameters(),
    )?
    .decoder()
    .video()?;

    let width = decoder.width();
    let height = decoder.height();
    let fps = input_stream.rate();

    let mut to_rgb_scaler = ScalerContext::get(
        decoder.format(),
        width,
        height,
        Pixel::RGB24,
        width,
        height,
        Flags::BILINEAR,
    )?;

    let mut to_yuv_scaler = ScalerContext::get(
        Pixel::RGB24,
        width,
        height,
        Pixel::YUV420P,
        width,
        height,
        Flags::BILINEAR,
    )?;

    let mut octx = output(&output_path)?;
    let codec = ffmpeg::encoder::find(ffmpeg::codec::Id::H264).unwrap();
    let mut encoder = ffmpeg::codec::context::Context::new_with_codec(codec)
        .encoder()
        .video()?;

    encoder.set_width(width);
    encoder.set_height(height);
    encoder.set_format(Pixel::YUV420P);
    let encoder_time_base = fps.invert();
    encoder.set_time_base(encoder_time_base);
    encoder.set_frame_rate(Some(fps));
    encoder.set_gop(12);
    encoder.set_bit_rate(2_000_000);

    if octx
        .format()
        .flags()
        .contains(ffmpeg::format::Flags::GLOBAL_HEADER)
    {
        encoder.set_flags(ffmpeg::codec::Flags::GLOBAL_HEADER);
    }

    let mut opts = ffmpeg::Dictionary::new();
    opts.set("preset", "ultrafast");
    opts.set("tune", "zerolatency");
    let mut encoder = encoder.open_as_with(codec, opts)?;

    let output_stream_index = {
        let mut ost = octx.add_stream(codec)?;
        ost.set_parameters(&encoder);
        ost.index()
    };

    octx.write_header()?;

    let font = FontArc::try_from_slice(dejavu::sans::regular()).unwrap();
    let text_scale = PxScale::from(14.0);

    println!("Processing frames...");

    let mut frame_count = 0;
    let mut pts_counter: i64 = 0;
    let mut active_tracks: HashMap<
        u64,
        (ObjectClass, OrientedBoundingBox, f32),
    > = HashMap::new();

    let mut process_and_encode_frame =
        |decoded_frame: &VideoFrame,
         encoder: &mut ffmpeg::codec::encoder::Video,
         octx: &mut ffmpeg::format::context::Output,
         pts: &mut i64,
         frames: &mut usize|
         -> Result<()> {
            *frames += 1;

            let mut rgb_frame = VideoFrame::empty();
            to_rgb_scaler.run(decoded_frame, &mut rgb_frame)?;

            let rgb_img =
                RgbImage::from_raw(width, height, rgb_frame.data(0).to_vec())
                    .unwrap();
            let mut dynamic_frame = DynamicImage::ImageRgb8(rgb_img);

            let events = pipeline.process_frame(dynamic_frame.clone())?;

            active_tracks.clear();
            for event in events {
                if let InstinctEvent::ObstacleDetected {
                    id,
                    class,
                    obb,
                    confidence,
                } = event
                {
                    active_tracks.insert(id, (class, obb, confidence));
                }
            }
            while let Ok(_) = pipeline.buffalo_rx.try_recv() {}

            let frame_buffer = dynamic_frame.as_mut_rgb8().unwrap();
            for (id, (class, obb, confidence)) in &active_tracks {
                let tlwh: [f32; 4] = obb.to_tlwh();
                let (x, y) =
                    (tlwh[0].max(0.0) as i32, tlwh[1].max(0.0) as i32);
                let (w, h) = (tlwh[2] as u32, tlwh[3] as u32);

                let color = match class {
                    ObjectClass::Person | ObjectClass::Animal => {
                        Rgb([255, 0, 0])
                    },
                    ObjectClass::Bicycle
                    | ObjectClass::Car
                    | ObjectClass::Motorcycle
                    | ObjectClass::Bus
                    | ObjectClass::Truck
                    | ObjectClass::Train => Rgb([0, 255, 0]),
                    ObjectClass::Pole
                    | ObjectClass::PowerLine
                    | ObjectClass::TrafficSign
                    | ObjectClass::TrafficLight => Rgb([255, 165, 0]),
                    ObjectClass::Building
                    | ObjectClass::Tree
                    | ObjectClass::Wall
                    | ObjectClass::Fence
                    | ObjectClass::StopSign
                    | ObjectClass::FireHydrant
                    | ObjectClass::ParkingMeter
                    | ObjectClass::Bench => Rgb([0, 0, 255]),
                    ObjectClass::Backpack
                    | ObjectClass::Umbrella
                    | ObjectClass::Handbag
                    | ObjectClass::Suitcase
                    | ObjectClass::TrashCan => Rgb([128, 0, 128]),
                    ObjectClass::Unknown => Rgb([100, 100, 100]),
                };

                draw_hollow_rect_mut(
                    frame_buffer,
                    Rect::at(x, y).of_size(w, h),
                    color,
                );
                draw_text_mut(
                    frame_buffer,
                    Rgb([255, 255, 255]),
                    x,
                    (y - 16).max(0),
                    text_scale,
                    &font,
                    &format!(
                        "{:?} (#{}) {:.0}%",
                        class,
                        id,
                        confidence * 100.0
                    ),
                );
            }

            let mut final_rgb_frame =
                VideoFrame::new(Pixel::RGB24, width, height);
            final_rgb_frame
                .data_mut(0)
                .copy_from_slice(frame_buffer.as_raw());

            let mut yuv_output_frame =
                VideoFrame::new(Pixel::YUV420P, width, height);
            to_yuv_scaler.run(&final_rgb_frame, &mut yuv_output_frame)?;

            yuv_output_frame.set_pts(Some(*pts));
            *pts += 1;

            encoder.send_frame(&yuv_output_frame)?;
            let mut encoded_packet = ffmpeg::Packet::empty();

            while encoder.receive_packet(&mut encoded_packet).is_ok() {
                encoded_packet.set_stream(output_stream_index);
                let out_tb =
                    octx.stream(output_stream_index).unwrap().time_base();
                encoded_packet.rescale_ts(encoder_time_base, out_tb);
                encoded_packet.write_interleaved(octx)?;
            }

            Ok(())
        };

    for (_, packet) in ictx.packets() {
        if packet.stream() == video_stream_index
            && decoder.send_packet(&packet).is_ok()
        {
            let mut decoded_frame = VideoFrame::empty();
            while decoder.receive_frame(&mut decoded_frame).is_ok() {
                process_and_encode_frame(
                    &decoded_frame,
                    &mut encoder,
                    &mut octx,
                    &mut pts_counter,
                    &mut frame_count,
                )?;
            }
        }
    }

    let _ = decoder.send_eof();
    let mut decoded_frame = VideoFrame::empty();
    while decoder.receive_frame(&mut decoded_frame).is_ok() {
        process_and_encode_frame(
            &decoded_frame,
            &mut encoder,
            &mut octx,
            &mut pts_counter,
            &mut frame_count,
        )?;
    }

    encoder.send_eof()?;
    let mut encoded_packet = ffmpeg::Packet::empty();
    while encoder.receive_packet(&mut encoded_packet).is_ok() {
        encoded_packet.set_stream(output_stream_index);
        let out_tb = octx.stream(output_stream_index).unwrap().time_base();
        encoded_packet.rescale_ts(encoder_time_base, out_tb);
        encoded_packet.write_interleaved(&mut octx)?;
    }

    octx.write_trailer()?;
    println!("Done: {} frames -> {:?}", frame_count, output_path);

    assert!(frame_count > 0);
    Ok(())
}
