mod memory;
use memory::Memory;

fn main() {
    let mut memory: Memory = Memory::new();

    memory.write(0x00, 10);
    let test = memory.read(0x00);
    println!("test is {}", test);
}
