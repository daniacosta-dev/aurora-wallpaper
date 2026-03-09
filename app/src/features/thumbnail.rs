use std::path::{Path, PathBuf};

use gstreamer::prelude::*;
use gstreamer_app::AppSink;
use gstreamer_video::VideoFormat;

#[derive(Debug)]
pub enum ThumbnailError {
    GstInit(gstreamer::glib::Error),
    PipelineError(String),
    NoFrame,
    Io(std::io::Error),
}

impl std::fmt::Display for ThumbnailError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThumbnailError::GstInit(e) => write!(f, "GStreamer init error: {e}"),
            ThumbnailError::PipelineError(s) => write!(f, "Pipeline error: {s}"),
            ThumbnailError::NoFrame => write!(f, "Could not extract frame from video"),
            ThumbnailError::Io(e) => write!(f, "I/O error: {e}"),
        }
    }
}

/// Extract a thumbnail from `video_path` at ~2 seconds and save it as PNG.
///
/// Output path: `thumbnails_dir/<wallpaper_id>.png`
///
/// Returns the path to the saved thumbnail on success.
pub fn extract_thumbnail(
    video_path: &Path,
    thumbnails_dir: &Path,
    wallpaper_id: u64,
) -> Result<PathBuf, ThumbnailError> {
    // Init GStreamer (safe to call multiple times).
    gstreamer::init().map_err(ThumbnailError::GstInit)?;

    std::fs::create_dir_all(thumbnails_dir).map_err(ThumbnailError::Io)?;

    let output_path = thumbnails_dir.join(format!("{wallpaper_id}.png"));

    // Build pipeline: decode video → convert to RGB → appsink
    let uri = format!(
        "file://{}",
        video_path.canonicalize()
            .unwrap_or_else(|_| video_path.to_owned())
            .display()
    );

    let pipeline_str = format!(
        "uridecodebin uri=\"{uri}\" ! videoconvert ! videoscale ! \
         video/x-raw,format=RGB,width=320,height=180,pixel-aspect-ratio=1/1 ! \
         appsink name=sink max-buffers=1 drop=true"
    );

    let pipeline = gstreamer::parse::launch(&pipeline_str)
        .map_err(|e| ThumbnailError::PipelineError(e.to_string()))?
        .downcast::<gstreamer::Pipeline>()
        .map_err(|_| ThumbnailError::PipelineError("Not a pipeline".into()))?;

    let appsink = pipeline
        .by_name("sink")
        .ok_or_else(|| ThumbnailError::PipelineError("No appsink".into()))?
        .downcast::<AppSink>()
        .map_err(|_| ThumbnailError::PipelineError("Not an appsink".into()))?;

    // Seek to 2 seconds after starting.
    pipeline
        .set_state(gstreamer::State::Playing)
        .map_err(|e| ThumbnailError::PipelineError(format!("Could not play: {e:?}")))?;

    // Wait until we can seek (pipeline is running).
    let _ = pipeline.state(gstreamer::ClockTime::from_seconds(5));

    let seek_pos = gstreamer::ClockTime::from_seconds(2);
    let _ = pipeline.seek_simple(
        gstreamer::SeekFlags::FLUSH | gstreamer::SeekFlags::KEY_UNIT,
        seek_pos,
    );

    // Pull one sample.
    let sample = appsink
        .pull_sample()
        .map_err(|_| ThumbnailError::NoFrame)?;

    let buffer = sample.buffer().ok_or(ThumbnailError::NoFrame)?;
    let map = buffer.map_readable().map_err(|_| ThumbnailError::NoFrame)?;
    let data = map.as_slice();

    // Save as PNG using only std (manual PNG encode via the `png` crate is ideal,
    // but here we use a simple PPM→PNG approach via the image crate).
    save_rgb_as_png(data, 320, 180, &output_path)?;

    pipeline
        .set_state(gstreamer::State::Null)
        .map_err(|e| ThumbnailError::PipelineError(format!("Could not stop: {e:?}")))?;

    Ok(output_path)
}

/// Save raw RGB (3 bytes/pixel) buffer as PNG.
fn save_rgb_as_png(
    data: &[u8],
    width: u32,
    height: u32,
    path: &Path,
) -> Result<(), ThumbnailError> {
    use std::fs::File;
    use std::io::BufWriter;

    let file = File::create(path).map_err(ThumbnailError::Io)?;
    let writer = BufWriter::new(file);

    let mut encoder = png::Encoder::new(writer, width, height);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);

    let mut png_writer = encoder
        .write_header()
        .map_err(|e: png::EncodingError| ThumbnailError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

    png_writer
        .write_image_data(data)
        .map_err(|e: png::EncodingError| ThumbnailError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

    Ok(())
}