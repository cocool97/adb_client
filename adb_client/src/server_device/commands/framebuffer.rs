use std::{
    io::{Read, Seek, Write},
    iter::Map,
    path::Path,
    slice::ChunksExact,
};

use byteorder::{LittleEndian, ReadBytesExt};
use image::{ImageBuffer, ImageFormat, Rgba};

use crate::{models::AdbServerCommand, utils, ADBServerDevice, Result, RustADBError};

type U32ChunkIter<'a> = Map<ChunksExact<'a, u8>, fn(&[u8]) -> Result<u32>>;

fn read_next(chunks: &mut U32ChunkIter) -> Result<u32> {
    chunks
        .next()
        .ok_or(RustADBError::FramebufferConversionError)?
}

#[derive(Debug)]
struct FrameBufferInfoV1 {
    pub _bpp: u32,
    pub size: u32,
    pub width: u32,
    pub height: u32,
    pub _red_offset: u32,
    pub _red_length: u32,
    pub _blue_offset: u32,
    pub _blue_length: u32,
    pub _green_offset: u32,
    pub _green_length: u32,
    pub _alpha_offset: u32,
    pub _alpha_length: u32,
}

impl TryFrom<[u8; std::mem::size_of::<Self>()]> for FrameBufferInfoV1 {
    type Error = RustADBError;

    fn try_from(
        value: [u8; std::mem::size_of::<Self>()],
    ) -> std::result::Result<Self, Self::Error> {
        let mut chunks: U32ChunkIter = value.chunks_exact(4).map(utils::u32_from_le);

        Ok(Self {
            _bpp: read_next(&mut chunks)?,
            size: read_next(&mut chunks)?,
            width: read_next(&mut chunks)?,
            height: read_next(&mut chunks)?,
            _red_offset: read_next(&mut chunks)?,
            _red_length: read_next(&mut chunks)?,
            _blue_offset: read_next(&mut chunks)?,
            _blue_length: read_next(&mut chunks)?,
            _green_offset: read_next(&mut chunks)?,
            _green_length: read_next(&mut chunks)?,
            _alpha_offset: read_next(&mut chunks)?,
            _alpha_length: read_next(&mut chunks)?,
        })
    }
}

#[derive(Debug)]
struct FrameBufferInfoV2 {
    pub _bpp: u32,
    pub _color_space: u32,
    pub size: u32,
    pub width: u32,
    pub height: u32,
    pub _red_offset: u32,
    pub _red_length: u32,
    pub _blue_offset: u32,
    pub _blue_length: u32,
    pub _green_offset: u32,
    pub _green_length: u32,
    pub _alpha_offset: u32,
    pub _alpha_length: u32,
}

impl TryFrom<[u8; std::mem::size_of::<Self>()]> for FrameBufferInfoV2 {
    type Error = RustADBError;

    fn try_from(
        value: [u8; std::mem::size_of::<Self>()],
    ) -> std::result::Result<Self, Self::Error> {
        let mut chunks: U32ChunkIter = value.chunks_exact(4).map(utils::u32_from_le);

        Ok(Self {
            _bpp: read_next(&mut chunks)?,
            _color_space: read_next(&mut chunks)?,
            size: read_next(&mut chunks)?,
            width: read_next(&mut chunks)?,
            height: read_next(&mut chunks)?,
            _red_offset: read_next(&mut chunks)?,
            _red_length: read_next(&mut chunks)?,
            _blue_offset: read_next(&mut chunks)?,
            _blue_length: read_next(&mut chunks)?,
            _green_offset: read_next(&mut chunks)?,
            _green_length: read_next(&mut chunks)?,
            _alpha_offset: read_next(&mut chunks)?,
            _alpha_length: read_next(&mut chunks)?,
        })
    }
}

impl ADBServerDevice {
    /// Dump framebuffer of this device into given ['path']
    /// Big help from source code (<https://android.googlesource.com/platform/system/adb/+/refs/heads/main/framebuffer_service.cpp>)
    pub fn framebuffer<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let img = self.framebuffer_inner()?;
        Ok(img.save(path.as_ref())?)
    }

    /// Dump framebuffer of this device and return corresponding bytes.
    ///
    /// Output data format is currently only `PNG`.
    pub fn framebuffer_bytes<W: Write + Seek>(&mut self, mut writer: W) -> Result<()> {
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

                let h: FrameBufferInfoV1 = buf.try_into()?;

                let mut data = vec![
                    0_u8;
                    h.size
                        .try_into()
                        .map_err(|_| RustADBError::ConversionError)?
                ];
                self.get_transport_mut()
                    .get_raw_connection()?
                    .read_exact(&mut data)?;

                Ok(
                    ImageBuffer::<Rgba<u8>, Vec<u8>>::from_vec(h.width, h.height, data)
                        .ok_or_else(|| RustADBError::FramebufferConversionError)?,
                )
            }
            // RGBX_8888
            2 => {
                let mut buf = [0u8; std::mem::size_of::<FrameBufferInfoV2>()];

                self.get_transport_mut()
                    .get_raw_connection()?
                    .read_exact(&mut buf)?;

                let h: FrameBufferInfoV2 = buf.try_into()?;

                let mut data = vec![
                    0_u8;
                    h.size
                        .try_into()
                        .map_err(|_| RustADBError::ConversionError)?
                ];
                self.get_transport_mut()
                    .get_raw_connection()?
                    .read_exact(&mut data)?;

                Ok(
                    ImageBuffer::<Rgba<u8>, Vec<u8>>::from_vec(h.width, h.height, data)
                        .ok_or_else(|| RustADBError::FramebufferConversionError)?,
                )
            }
            v => Err(RustADBError::UnimplementedFramebufferImageVersion(v)),
        }
    }
}
