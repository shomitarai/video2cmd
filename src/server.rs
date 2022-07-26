use opencv::prelude::*;
use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, TcpStream},
};

use crate::{image, opencv_wrapper};
use crate::{IMAGE_HEIGHT, IMAGE_WIDTH};

pub struct Listener {
    listener: TcpListener,
}

impl Listener {
    pub async fn new() -> Listener {
        Listener {
            listener: TcpListener::bind("127.0.0.1:3456").await.unwrap(),
        }
    }

    pub async fn run(&mut self) -> tokio::io::Result<()> {
        let (mut socket, _) = self.listener.accept().await.unwrap();

        let mut cam = opencv_wrapper::get_cam().unwrap();
        loop {
            // let mut frame = Mat::default();
            // cam.read(&mut frame).unwrap();
            // let compressed_frame = opencv_wrapper::get_compressed_frame(
            //     &frame,
            //     IMAGE_HEIGHT as i32,
            //     IMAGE_WIDTH as i32,
            // )
            // .unwrap();

            // let bytes = compressed_frame.data_bytes().unwrap();

            // socket.write_all(bytes).await;
            socket.write_all(b"aaa");
        }

        Ok(())
    }
}
