pub struct ADBUsbMessageHeader {
    command: u32,     /* command identifier constant      */
    arg0: u32,        /* first argument                   */
    arg1: u32,        /* second argument                  */
    data_length: u32, /* length of payload (0 is allowed) */
    data_crc32: u32,  /* crc32 of data payload            */
    magic: u32,       /* command ^ 0xffffffff             */
    payload: Vec<u8>,
}

impl ADBUsbMessageHeader {
    pub fn new(command: u32, arg0: u32, arg1: u32, data: Vec<u8>) -> Self {
        Self {
            command,
            arg0,
            arg1,
            data_length: data.len() as u32,
            data_crc32: data.iter().map(|&x| x as u32).sum(),
            magic: command ^ 0xFFFFFFFF,
            payload: data,
        }
    }

    pub fn to_bytes(&self) -> [u8; 24] {
        let mut result = [0u8; 24];
        let mut offset = 0;

        let command_bytes = self.command.to_le_bytes();
        result[offset..offset + 4].copy_from_slice(&command_bytes);
        offset += 4;

        let arg0_bytes = self.arg0.to_le_bytes();
        result[offset..offset + 4].copy_from_slice(&arg0_bytes);
        offset += 4;

        let arg1_bytes = self.arg1.to_le_bytes();
        result[offset..offset + 4].copy_from_slice(&arg1_bytes);
        offset += 4;

        let data_length_bytes = self.data_length.to_le_bytes();
        result[offset..offset + 4].copy_from_slice(&data_length_bytes);
        offset += 4;

        let data_crc32_bytes = self.data_crc32.to_le_bytes();
        result[offset..offset + 4].copy_from_slice(&data_crc32_bytes);
        offset += 4;

        let magic_bytes = self.magic.to_le_bytes();
        result[offset..offset + 4].copy_from_slice(&magic_bytes);

        result
    }

    pub fn to_payload(self) -> Vec<u8> {
        self.payload
    }
}
