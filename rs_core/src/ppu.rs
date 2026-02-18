pub struct Ppu {
    ram: [u8; 8192],
}

impl Ppu {
    pub fn new() -> Self {
        Ppu { ram: [0; 8192] }
    }
    pub fn load_patterns(&mut self, data: Vec<u8>) {}
    pub fn write(&mut self, addr: u16) {}
    pub fn read(&self, addr: u16) -> u8 {
        0
    }
}
