use std::io::{self, Cursor, Read};

const PRG_UNIT: usize = 16 * 1024;
const CHR_UNIT: usize = 8 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mirroring {
    Horizontal,
    Vertical,
    SingleScreen,
    FourScreen,
    // There are other uncommon ones TODO
}

pub struct Cartridge {
    // CPU DATA
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
    // 8 KB cartridge work RAM mapped at $6000-$7FFF
    wram: Vec<u8>,
    mirroring: Mirroring,
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

        /*
        76543210
        ||||||||
        |||||||+- Nametable arrangement: 0: vertical arrangement ("horizontal mirrored") (CIRAM A10 = PPU A11)
        |||||||                          1: horizontal arrangement ("vertically mirrored") (CIRAM A10 = PPU A10)
        ||||||+-- 1: Cartridge contains battery-backed PRG RAM ($6000-7FFF) or other persistent memory
        |||||+--- 1: 512-byte trainer at $7000-$71FF (stored before PRG data)
        ||||+---- 1: Alternative nametable layout
        ++++----- Lower nybble of mapper number
        */
        let flags6 = header[6];
        let mirroring = if flags6 & 0x08 != 0 {
            Mirroring::FourScreen
        } else if flags6 & 0x01 != 0 {
            Mirroring::Vertical
        } else {
            Mirroring::Horizontal
        };

        let trainer_present = flags6 & 0x04 != 0;

        let offset = 16 // Header size
            + if trainer_present {
                512 // Trainer size
            } else {
                0
        }; // Add 512 if trainer is present

        let mut cur = Cursor::new(&bytes[offset..]);
        let mut prg_rom = vec![0; prg_rom_size];
        let mut chr_rom = vec![0; chr_rom_size];

        cur.read_exact(&mut prg_rom).map_err(|e| {
            std::io::Error::new(
                e.kind(),
                format!("ROM too small for PRG: wanted {} bytes ({e})", prg_rom_size),
            )
        })?;
        cur.read_exact(&mut chr_rom).map_err(|e| {
            std::io::Error::new(
                e.kind(),
                format!("ROM too small for CHR: wanted {} bytes ({e})", chr_rom_size),
            )
        })?;

        Ok(Cartridge {
            prg_rom,
            chr_rom,
            wram: vec![0; 0x2000],
            mirroring,
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
        &self.chr_rom
    }

    pub fn mirroring(&self) -> Mirroring {
        self.mirroring
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
