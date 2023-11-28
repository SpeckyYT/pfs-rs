pub const ARCHIVE_MAGIC: [u8; 2] = [ 0x70, 0x66 ];

pub const HEADER_SIZE: usize = 11;

#[derive(Debug, Default, Clone)]
pub struct Entry {
    // pub local_path: String,
    pub path: String,
    // pub position: u8,
    pub offset: u32,
    pub size: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct Header {
    pub magic: [u8; 2],
    pub pack_version: u8,
    pub index_size: u32,
    pub file_count: u32,
}

impl Default for Header {
    fn default() -> Self {
        Self {
            magic: ARCHIVE_MAGIC,
            pack_version: b'8', 
            index_size: 0,
            file_count: 0,
        }
    }
}

impl Header {
    pub fn from_bytes(bytes: &[u8; HEADER_SIZE]) -> Self {

        let magic = bytes[0..2].try_into().unwrap();
        let pack_version = bytes[2];
        let index_size = u32::from_le_bytes(bytes[3..7].try_into().unwrap());
        let file_count = u32::from_le_bytes(bytes[7..11].try_into().unwrap());

        Header {
            magic,
            pack_version,
            index_size,
            file_count,
        }
    }

    pub fn to_bytes(self) -> [u8; HEADER_SIZE] {
        let mut bytes = [0; HEADER_SIZE];

        bytes[0..2].copy_from_slice(&self.magic);
        bytes[2] = self.pack_version;
        bytes[3..7].copy_from_slice(&self.index_size.to_le_bytes());
        bytes[7..11].copy_from_slice(&self.file_count.to_le_bytes());

        bytes
    }

    pub fn has_magic(&self) -> bool {
        self.magic == ARCHIVE_MAGIC
    }
}
