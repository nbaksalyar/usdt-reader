//! USDT probes reader.

use std::fs;
use std::io::{Cursor, Read};
use std::path::Path;

use goblin::elf::Elf;
use goblin::Object;

const NT_STAPSDT: u32 = 3;

#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    GoblinError(goblin::error::Error),
    UnsupportedObjectType(String),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IoError(e)
    }
}

impl From<goblin::error::Error> for Error {
    fn from(e: goblin::error::Error) -> Self {
        Error::GoblinError(e)
    }
}

pub struct Context {}

impl Context {
    pub fn new_from_bin<P>(bin_path: &P) -> Result<Context, Error>
    where
        P: AsRef<Path>,
    {
        let buffer = fs::read(bin_path.as_ref())?;
        let obj = Object::parse(&buffer)?;

        match obj {
            Object::Elf(elf) => Self::new_from_elf(&buffer, elf),
            obj_type => return Err(Error::UnsupportedObjectType(format!("{:?}", obj_type))),
        }
    }

    pub fn new_from_pid(pid: u64) {}

    pub fn new_from_elf(data: &[u8], elf: Elf) -> Result<Context, Error> {
        // See if this is a SystemTap probe.
        let probes_section = elf.section_headers.iter().find(|shdr| {
            let section_name = elf.shdr_strtab.get_at(shdr.sh_name).unwrap_or("");
            section_name == ".probes"
        });

        let stap_note = elf.iter_note_sections(data, Some(".note.stapsdt"));

        let stap_note = stap_note.unwrap().next().unwrap().unwrap();
        assert!(stap_note.n_type == NT_STAPSDT && stap_note.name == "stapsdt");

        // parse stap note
        let mut desc = Cursor::new(stap_note.desc);
        if elf.is_64 {
            let pc_addr = read_u64_from_desc(&mut desc, &elf);
            let sh_addr = read_u64_from_desc(&mut desc, &elf);
            let semaphore_addr = read_u64_from_desc(&mut desc, &elf);
            dbg!(pc_addr, sh_addr, semaphore_addr);
        } else {
            let pc_addr = read_u32_from_desc(&mut desc, &elf);
            let sh_addr = read_u32_from_desc(&mut desc, &elf);
            let semaphore_addr = read_u32_from_desc(&mut desc, &elf);
            dbg!(pc_addr, sh_addr, semaphore_addr);
        }

        Ok(Context {})
    }

    pub fn probes() {}
}

fn read_u64_from_desc(desc: &mut Cursor<&[u8]>, elf: &Elf) -> u64 {
    let mut bytes = [0u8; 8];
    desc.read_exact(&mut bytes).unwrap();
    if elf.little_endian {
        u64::from_le_bytes(bytes)
    } else {
        u64::from_be_bytes(bytes)
    }
}

fn read_u32_from_desc(desc: &mut Cursor<&[u8]>, elf: &Elf) -> u32 {
    let mut bytes = [0u8; 4];
    desc.read_exact(&mut bytes).unwrap();
    if elf.little_endian {
        u32::from_le_bytes(bytes)
    } else {
        u32::from_be_bytes(bytes)
    }
}
