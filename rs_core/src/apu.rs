pub struct Apu {}

impl Apu {
    pub fn write(&mut self, addr: u16) {}
    pub fn read(&self, addr: u16) -> u8 {
        // TODO handle test space
        0
    }
}
