use crate::cart::Mirroring;

const NAME_TABLE_SIZE: usize = 0x03C0;
const ATTR_TABLE_SIZE: usize = 0x0040;
const NAME_ATTR_TABLE_COUNT: usize = 4;
const PALETTE_SIZE: usize = 0x020;
const BG_PALETTE_BASE_ADDR: u16 = 0x3F00;
const SPRITE_PALETTE_BASE_ADDR: u16 = 0x3F10;
const CHR_SIZE: usize = 0x2000;

pub(crate) struct PpuBus {
    chr: [u8; CHR_SIZE],
    name_tables: [u8; (NAME_TABLE_SIZE + ATTR_TABLE_SIZE) * NAME_ATTR_TABLE_COUNT],
    palette: [u8; PALETTE_SIZE],
    mirroring: Mirroring,
}

impl PpuBus {
    pub fn new() -> Self {
        PpuBus {
            chr: [0; CHR_SIZE],
            name_tables: [0; (NAME_TABLE_SIZE + ATTR_TABLE_SIZE) * NAME_ATTR_TABLE_COUNT],
            palette: [0; PALETTE_SIZE],
            mirroring: Mirroring::Vertical,
        }
    }

    #[inline]
    pub fn load(&mut self, chr_data: &[u8], mirroring: Mirroring) {
        let len = chr_data.len().min(CHR_SIZE);
        self.chr[..len].copy_from_slice(&chr_data[..len]);
        self.mirroring = mirroring;
    }

    #[inline]
    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => self.chr[addr as usize],
            0x2000..=0x3EFF => self.name_tables[(addr & 0x0FFF) as usize],
            0x3F00..=0x3FFF => self.palette[(addr & 0x001F) as usize], // TODO check entry
            _ => 0x0,
        }
    }

    #[inline]
    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000..=0x1FFF => self.chr[addr as usize] = val,
            0x2000..=0x3EFF => self.name_tables[(addr & 0x0FFF) as usize] = val,
            0x3F00..=0x3FFF => self.palette[(addr & 0x001F) as usize] = val, // TODO check entry
            _ => (),
        };
    }

    pub fn palette_lookup(&self, sprite: bool, idx: u8) -> [u8; 4] {
        let start = if sprite {
            SPRITE_PALETTE_BASE_ADDR
        } else {
            BG_PALETTE_BASE_ADDR
        } + ((idx & 0x03) as u16 * 4);

        [
            self.read(start),
            self.read(start + 1),
            self.read(start + 2),
            self.read(start + 3),
        ]
    }
}
