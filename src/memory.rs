const MEM_SIZE: usize = 4 * 1024;
const MEM_RANGE: std::ops::Range<usize> = 0x000..0xFFF;

pub type Cell = u8;
pub type Address = usize;
pub struct Memory([Cell; MEM_SIZE]);

fn valid_address(addr: Address) -> bool {
    MEM_RANGE.contains(&addr)
}

impl Memory {
    pub fn new() -> Memory {
        Memory([0x00; MEM_SIZE])
    }

    pub fn read(&self, address: Address) -> Cell {
        if valid_address(address) {
            self.0[address]
        } else {
            0x00
        }
    }

    pub fn write(&mut self, address: Address, data: Cell) {
        if valid_address(address) {
            self.0[address] = data;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_out_of_range_returns_0x00() {
        let mut mem = Memory::new();

        for addr in MEM_RANGE {
            mem.write(addr, 1);
        }

        assert_eq!(1,  mem.read(0x000)); // Sanity check
        assert_eq!(0,  mem.read(0xFFF1)); // out of range

    }

    #[test]
    fn can_write() {
        let mut mem = Memory::new();

        mem.write(0x000, 1);
        mem.write(0x200, 10);
        mem.write(0x500, 15);
        mem.write(0xFFF1, 1); // out of range

        assert_eq!(1,  mem.read(0x00));
        assert_eq!(10, mem.read(0x200));
        assert_eq!(15, mem.read(0x500));
        assert_eq!(0,  mem.read(0xFFF1)); // out of range

    }
}
