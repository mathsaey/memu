#![feature(const_generics)]

mod generic;
use generic::memory::Memory;

const MEM_SIZE: usize = 4 * 1024;
const MEM_MAX:  usize = 0xFFF;

type MemCell = u8;


fn main() {
    let mut mem: Memory<MemCell, MEM_SIZE> = Memory::new(MEM_MAX);
}
