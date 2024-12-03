use std::{
    io::{Read, Seek, Write},
    path::Path,
};

use byteorder::{LittleEndian, ReadBytesExt};
use image::{ImageBuffer, ImageFormat, Rgba};

use crate::{
    models::{AdbServerCommand, FrameBufferInfoV1, FrameBufferInfoV2},
    ADBServerDevice, Result, RustADBError,
};

impl ADBServerDevice {
    /// Dump framebuffer of this device into given ['path']
    /// Big help from source code (<https://android.googlesource.com/platform/system/adb/+/refs/heads/main/framebuffer_service.cpp>)
    pub(crate) fn framebuffer<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let img = self.framebuffer_inner()?;
        Ok(img.save(path.as_ref())?)
    }

    /// Dump framebuffer of this device and return corresponding bytes.
    ///
    /// Output data format is currently only `PNG`.
    pub(crate) fn framebuffer_bytes<W: Write + Seek>(&mut self, mut writer: W) -> Result<()> {
        let img = self.framebuffer_inner()?;
        Ok(img.write_to(&mut writer, ImageFormat::Png)?)
    }

    /// Inner method requesting framebuffer from Android device
    fn framebuffer_inner(&mut self) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>> {
        let serial: String = self.identifier.clone();
        self.connect()?
            .send_adb_request(AdbServerCommand::TransportSerial(serial))?;

        self.get_transport_mut()
            .send_adb_request(AdbServerCommand::FrameBuffer)?;

        let version = self
            .get_transport_mut()
            .get_raw_connection()?
            .read_u32::<LittleEndian>()?;

        match version {
            // RGBA_8888
            1 => {
                let mut buf = [0u8; std::mem::size_of::<FrameBufferInfoV1>()];

                self.get_transport_mut()
                    .get_raw_connection()?
                    .read_exact(&mut buf)?;

                let framebuffer_info: FrameBufferInfoV1 = buf.try_into()?;

                let mut data = vec![
                    0_u8;
                    framebuffer_info
                        .size
                        .try_into()
                        .map_err(|_| RustADBError::ConversionError)?
                ];
                self.get_transport_mut()
                    .get_raw_connection()?
                    .read_exact(&mut data)?;

                Ok(ImageBuffer::<Rgba<u8>, Vec<u8>>::from_vec(
                    framebuffer_info.width,
                    framebuffer_info.height,
                    data,
                )
                .ok_or_else(|| RustADBError::FramebufferConversionError)?)
            }
            // RGBX_8888
            2 => {
                let mut buf = [0u8; std::mem::size_of::<FrameBufferInfoV2>()];

                self.get_transport_mut()
                    .get_raw_connection()?
                    .read_exact(&mut buf)?;

                let framebuffer_info: FrameBufferInfoV2 = buf.try_into()?;

                let mut data = vec![
                    0_u8;
                    framebuffer_info
                        .size
                        .try_into()
                        .map_err(|_| RustADBError::ConversionError)?
                ];
                self.get_transport_mut()
                    .get_raw_connection()?
                    .read_exact(&mut data)?;

                Ok(ImageBuffer::<Rgba<u8>, Vec<u8>>::from_vec(
                    framebuffer_info.width,
                    framebuffer_info.height,
                    data,
                )
                .ok_or_else(|| RustADBError::FramebufferConversionError)?)
            }
            v => Err(RustADBError::UnimplementedFramebufferImageVersion(v)),
        }
    }
}
