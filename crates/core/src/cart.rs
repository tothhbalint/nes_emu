use std::io;

const PRG_UNIT: usize = 16 * 1024;
const CHR_UNIT: usize = 8 * 1024;

pub struct Cartridge {
    // CPU DATA
    prg_rom: Vec<u8>,
    cart_ram: Vec<u8>,
    // 8 KB cartridge work RAM mapped at $6000-$7FFF
    wram: Vec<u8>,
}

fn parse_header(bytes: &Vec<u8>) -> std::io::Result<[u8; 16]> {
    bytes
        .get(0..16)
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "Rom too small"))?
        .try_into()
        .map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "Failed to convert header")
        })
}

impl Cartridge {
    pub fn from_bytes(bytes: &Vec<u8>) -> std::io::Result<Self> {
        let header: [u8; 16] = parse_header(bytes)?;

        if &header[0..4] != b"NES\x1A" {
            return Err(std::io::Error::new(
                io::ErrorKind::InvalidData,
                "Not a valid iNes file",
            ));
        }

        let prg_rom_size = header[4] as usize * PRG_UNIT;
        let chr_rom_size = header[5] as usize * CHR_UNIT;

        let trainer_present = header[6] & 0b100 != 0;
        let mut offset = 16;
        if trainer_present {
            offset += 512;
        }

        let prg_rom = bytes
            .get(offset..offset + prg_rom_size)
            .ok_or_else(|| {
                std::io::Error::new(io::ErrorKind::UnexpectedEof, "Rom too small for PRG size")
            })?
            .to_vec();

        offset += prg_rom_size;

        let cart_ram = bytes
            .get(offset..offset + chr_rom_size)
            .ok_or_else(|| {
                std::io::Error::new(io::ErrorKind::UnexpectedEof, "Rom too small for CHR size")
            })?
            .to_vec();

        Ok(Cartridge {
            prg_rom,
            cart_ram,
            wram: vec![0; 0x2000],
        })
    }

    /// Reads from the cartridge address space ($4020-$FFFF).
    pub fn read(&self, addr: usize) -> u8 {
        match addr {
            0x6000..=0x7FFF => self.wram[addr - 0x6000],
            0x8000..=0xFFFF => self.read_prg(addr),
            _ => 0,
        }
    }

    /// Writes to the cartridge address space ($4020-$FFFF). Writes to PRG ROM
    /// (and, for now, expansion) are ignored; $6000-$7FFF is cartridge WRAM.
    pub fn write(&mut self, addr: usize, data: u8) {
        if let 0x6000..=0x7FFF = addr {
            self.wram[addr - 0x6000] = data;
        }
    }

    pub fn read_prg(&self, addr: usize) -> u8 {
        let mut idx = addr % self.prg_rom.len();
        if self.prg_rom.len() == PRG_UNIT {
            idx = idx % PRG_UNIT;
        }

        self.prg_rom[idx]
    }

    /// The raw CHR ROM / pattern table data as parsed from the iNES file.
    pub fn chr(&self) -> &[u8] {
        &self.cart_ram
    }
    /// Dumps a range of CPU addresses mapped to PRG ROM
    /// start_addr and end_addr are CPU addresses (0x8000..0xFFFF)
    pub fn dump_prg_range(&self, start_addr: u16, end_addr: u16) {
        assert!(start_addr >= 0x8000 && end_addr <= 0xFFFF);
        let prg_len = self.prg_rom.len();

        for addr in start_addr..=end_addr {
            let mut idx = (addr - 0x8000) as usize;

            // Mirror 16 KB PRG ROM if necessary
            if prg_len == 16 * 1024 && idx >= 16 * 1024 {
                idx %= 16 * 1024;
            }

            // Print address and byte
            println!("{:04X}: {:02X}", addr, self.prg_rom[idx]);
        }
    }
}
