const MEM_SIZE: usize = 4 * 1024;

pub type Cell = u8; // Is this useful / needed?
pub struct Memory([Cell ; MEM_SIZE]);

impl Memory {
    pub fn create() -> Memory {
        Memory([0x00 ; MEM_SIZE])
    }
}
