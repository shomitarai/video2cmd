use opencv::{core::CV_8UC3, highgui, imgproc, prelude::*, videoio, Result};

use crate::{IMAGE_HEIGHT, IMAGE_WIDTH};

pub fn get_vec8(cam: &mut impl VideoCaptureTrait) -> Result<Vec<u8>> {
    let mut frame = Mat::default();
    cam.read(&mut frame).unwrap();

    let compressed_frame =
        get_compressed_frame(&frame, IMAGE_HEIGHT as i32, IMAGE_WIDTH as i32).unwrap();
    let v = compressed_frame.data_bytes().unwrap().to_vec();
    Ok(v)
}

pub fn get_cam() -> Result<impl VideoCaptureTrait> {
    opencv::opencv_branch_32! {
        let mut cam = videoio::VideoCapture::new_default(0)?; // 0 is the default camera
    }
    opencv::not_opencv_branch_32! {
        let cam = videoio::VideoCapture::new(0, videoio::CAP_ANY)?; // 0 is the default camera
    }
    let opened = videoio::VideoCapture::is_opened(&cam)?;
    if !opened {
        panic!("Unable to open default camera!");
    }
    Ok(cam)
}

pub fn get_compressed_frame(frame: &Mat, rows: i32, cols: i32) -> Result<Mat> {
    let mut compressed_frame = unsafe { Mat::new_rows_cols(rows, cols, CV_8UC3)? };

    let dsize = compressed_frame.size()?.clone();

    imgproc::resize(&frame, &mut compressed_frame, dsize, 0.0, 0.0, 1)?;

    Ok(compressed_frame)
}
