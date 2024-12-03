use std::io::{Cursor, Read, Write};

use byteorder::{LittleEndian, ReadBytesExt};
use image::{ImageBuffer, ImageFormat, Rgba};

use crate::{
    device::{adb_message_device::ADBMessageDevice, ADBTransportMessage, MessageCommand},
    models::{FrameBufferInfoV1, FrameBufferInfoV2},
    ADBMessageTransport, Result, RustADBError,
};

impl<T: ADBMessageTransport> ADBMessageDevice<T> {
    pub fn framebuffer<P: AsRef<std::path::Path>>(&mut self, path: P) -> Result<()> {
        let img = self.framebuffer_inner()?;
        Ok(img.save(path.as_ref())?)
    }

    pub fn framebuffer_bytes<W: Write + std::io::Seek>(&mut self, mut writer: W) -> Result<()> {
        let img = self.framebuffer_inner()?;
        Ok(img.write_to(&mut writer, ImageFormat::Png)?)
    }

    fn framebuffer_inner(&mut self) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>> {
        let message =
            ADBTransportMessage::new(MessageCommand::Open, 1, 0, b"framebuffer:\0".to_vec());

        let response = self.send_and_expect_okay(message)?;

        let local_id = response.header().arg1();
        let remote_id = response.header().arg0();

        let response = self.recv_and_reply_okay(local_id, remote_id)?;

        let mut payload_cursor = Cursor::new(response.payload());

        let version = payload_cursor.read_u32::<LittleEndian>()?;

        let img = match version {
            // RGBA_8888
            1 => {
                let mut buf = [0u8; std::mem::size_of::<FrameBufferInfoV1>()];

                payload_cursor.read_exact(&mut buf)?;

                let framebuffer_info: FrameBufferInfoV1 = buf.try_into()?;

                let mut framebuffer_data = Vec::new();
                payload_cursor.read_to_end(&mut framebuffer_data)?;

                loop {
                    if framebuffer_data.len() as u32 == framebuffer_info.size {
                        break;
                    }

                    let response = self.recv_and_reply_okay(local_id, remote_id)?;

                    framebuffer_data.extend_from_slice(&response.into_payload());

                    log::debug!(
                        "received framebuffer data. new size {}",
                        framebuffer_data.len()
                    );
                }

                ImageBuffer::<Rgba<u8>, Vec<u8>>::from_vec(
                    framebuffer_info.width,
                    framebuffer_info.height,
                    framebuffer_data,
                )
                .ok_or_else(|| RustADBError::FramebufferConversionError)?
            }
            // RGBX_8888
            2 => {
                let mut buf = [0u8; std::mem::size_of::<FrameBufferInfoV2>()];

                payload_cursor.read_exact(&mut buf)?;

                let framebuffer_info: FrameBufferInfoV2 = buf.try_into()?;

                let mut framebuffer_data = Vec::new();
                payload_cursor.read_to_end(&mut framebuffer_data)?;

                loop {
                    if framebuffer_data.len() as u32 == framebuffer_info.size {
                        break;
                    }

                    let response = self.recv_and_reply_okay(local_id, remote_id)?;

                    framebuffer_data.extend_from_slice(&response.into_payload());

                    log::debug!(
                        "received framebuffer data. new size {}",
                        framebuffer_data.len()
                    );
                }

                ImageBuffer::<Rgba<u8>, Vec<u8>>::from_vec(
                    framebuffer_info.width,
                    framebuffer_info.height,
                    framebuffer_data,
                )
                .ok_or_else(|| RustADBError::FramebufferConversionError)?
            }
            v => return Err(RustADBError::UnimplementedFramebufferImageVersion(v)),
        };

        let message = self.get_transport_mut().read_message()?;
        match message.header().command() {
            MessageCommand::Clse => Ok(img),
            c => Err(RustADBError::ADBRequestFailed(format!(
                "Wrong command received {}",
                c
            ))),
        }
    }
}
