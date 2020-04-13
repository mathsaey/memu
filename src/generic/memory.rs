extern crate num;
use num::PrimInt;

use std::ops::Range;

pub type Address = usize;

pub struct Memory<T: PrimInt, const SIZE: usize> {
    mem: [T; SIZE],
    range: Range<usize>
}

impl<T: PrimInt, const SIZE: usize> Memory<T, SIZE> {
    fn valid_address(&self, addr: Address) -> bool {
        self.range.contains(&addr)
    }

    pub fn new(max_addr: Address) -> Memory<T, SIZE> {
        Memory{
            mem: [T::zero(); SIZE],
            range: 0x000..(max_addr + 1)
        }
    }

    pub fn read(&self, address: Address) -> T {
        if self.valid_address(address) {
            self.mem[address]
        } else {
            T::zero()
        }
    }

    pub fn write(&mut self, address: Address, data: T) {
        if self.valid_address(address) {
            self.mem[address] = data;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const SIZE: usize = 4 * 1024;

    #[test]
    fn read_out_of_range_returns_0x00() {
        let mut mem: Memory<u8, SIZE> = Memory::new(0xFFF);

        for addr in mem.range.clone() {
            mem.write(addr, 1);
        }

        assert_eq!(1,  mem.read(0x000)); // Sanity check
        assert_eq!(1,  mem.read(0xFFF)); // Sanity check
        assert_eq!(0,  mem.read(0xFFF1)); // out of range
    }

    #[test]
    fn can_write() {
        let mut mem: Memory<u8, SIZE> = Memory::new(0xFFF);

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
