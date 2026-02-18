use std::io;

const PRG_UNIT: usize = 16 * 1024;
const CHR_UNIT: usize = 8 * 1024;

pub struct Cartridge {
    // CPU DATA
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
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

        println!("{}", header[4]);
        println!("{}", header[5]);

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

        let chr_rom = bytes
            .get(offset..offset + chr_rom_size)
            .ok_or_else(|| {
                std::io::Error::new(io::ErrorKind::UnexpectedEof, "Rom too small for CHR size")
            })?
            .to_vec();

        Ok(Cartridge { prg_rom, chr_rom })
    }

    pub fn read_mem(&self, addr: usize) -> u8 {
        //assert!(addr >= 0x8000, "Address below PRG ROM mapping");

        let mut idx = addr % self.prg_rom.len();
        if self.prg_rom.len() == PRG_UNIT {
            idx = idx % PRG_UNIT;
        }

        self.prg_rom[idx]
    }

    pub fn write(&mut self, addr: usize, data: u8) {
        //assert!(addr >= 0x8000, "Address below PRG ROM mapping");

        let mut idx = addr % self.prg_rom.len();
        if self.prg_rom.len() == PRG_UNIT {
            idx = idx % PRG_UNIT;
        }

        self.prg_rom[idx] = data;
    }

    pub fn get_ppu_data(&self) -> Vec<u8> {
        self.chr_rom.clone()
    }
}
