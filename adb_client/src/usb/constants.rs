pub const CMD_CNXN: u32 = 0x4e584e43;
pub const CMD_AUTH: u32 = 0x48545541;
pub const CMD_OPEN: u32 = 0x4e45504f;
pub const CMD_OKAY: u32 = 0x59414b4f;
pub const CMD_WRTE: u32 = 0x45545257;
pub const CMD_SYNC: u32 = 0x434e5953;
pub const CMD_CLSE: u32 = 0x45534c43;

pub const CONNECT_VERSION: u32 = 0x01000000;
pub const CONNECT_MAXDATA: u32 = 4096;
pub const CONNECT_PAYLOAD: &str = "host::\0";
