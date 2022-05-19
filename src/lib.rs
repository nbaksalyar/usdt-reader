//! USDT probes reader.

mod error;

use std::io::{BufRead, Cursor, Read};

use goblin::elf::note::Note;
use goblin::elf::Elf;
use goblin::Object;

pub use error::Error;

const NT_STAPSDT: u32 = 3;

#[derive(Debug)]
pub struct Location {}

#[derive(Debug)]
pub struct Probe {
    /// Probe name.
    pub probe_name: String,
    /// Provider name.
    pub provider_name: String,
    /// Argument format.
    pub args_str: String,
    /// Probe PC address.
    pub probe_addr: u64,
    /// Link-time sh_addr of .stapsdt.base section
    pub sh_addr: u64,
    /// Link-time address of the semaphore variable.
    /// Zero if the probe does not have an associated semaphore.
    pub semaphore: u64,
    /// Semaphore address adjusted to sh_addr.
    pub semaphore_offset: u64,
}

impl Probe {
    pub fn arguments() {
        todo!()
    }
}

pub struct Context<'a> {
    data: &'a [u8],
    elf: Elf<'a>,
}

impl<'a> Context<'a> {
    /// Creates a new context from an object file data.
    pub fn new(data: &'a [u8]) -> Result<Context, Error> {
        let obj = Object::parse(&data)?;

        match obj {
            Object::Elf(elf) => Self::new_from_elf(data, elf),
            obj_type => return Err(Error::UnsupportedObjectType(format!("{:?}", obj_type))),
        }
    }

    pub fn new_from_elf(data: &'a [u8], elf: Elf<'a>) -> Result<Context<'a>, Error> {
        Ok(Context { elf, data })
    }

    pub fn probes(&'a self) -> Result<impl Iterator<Item = Result<Probe, Error>> + 'a, Error> {
        // See if this is a SystemTap probe.
        let probes_section = self.elf.section_headers.iter().find(|shdr| {
            let section_name = self.elf.shdr_strtab.get_at(shdr.sh_name).unwrap_or("");
            section_name == ".probes"
        });

        let notes_iter = self
            .elf
            .iter_note_sections(&self.data, Some(".note.stapsdt"))
            .unwrap() // FIXME: gracefully handle an empty note iter
            .map(move |p| Self::parse_probe(&self.elf, p));

        Ok(notes_iter)
    }

    fn parse_probe(
        elf: &'a Elf,
        note_desc: Result<Note, goblin::error::Error>,
    ) -> Result<Probe, Error> {
        let stap_note = note_desc?;
        assert!(stap_note.n_type == NT_STAPSDT && stap_note.name == "stapsdt");

        // parse stap note
        let mut desc = Cursor::new(stap_note.desc);

        let pc_addr = read_usize_from_desc(&mut desc, &elf)?;
        let sh_addr = read_usize_from_desc(&mut desc, &elf)?;
        let semaphore_addr = read_usize_from_desc(&mut desc, &elf)?;

        let mut provider_name = Vec::default();
        desc.read_until(b'\0', &mut provider_name)?;
        let _ = provider_name.pop();

        let mut probe_name = Vec::default();
        desc.read_until(b'\0', &mut probe_name)?;
        let _ = probe_name.pop();

        let mut arg_format = Vec::default();
        desc.read_until(b'\0', &mut arg_format)?;
        let _ = arg_format.pop();

        let p = Probe {
            probe_name: String::from_utf8(provider_name)?,
            provider_name: String::from_utf8(probe_name)?,
            args_str: String::from_utf8(arg_format)?,
            probe_addr: pc_addr,
            sh_addr,
            semaphore: semaphore_addr,
            semaphore_offset: semaphore_addr,
        };

        Ok(p)
    }
}

/// Reads an 'address size' number of bytes from the note description.
fn read_usize_from_desc(desc: &mut Cursor<&[u8]>, elf: &Elf) -> Result<u64, Error> {
    Ok(if elf.is_64 {
        let mut bytes = [0u8; 8];
        desc.read_exact(&mut bytes)?;
        if elf.little_endian {
            u64::from_le_bytes(bytes)
        } else {
            u64::from_be_bytes(bytes)
        }
    } else {
        let mut bytes = [0u8; 4];
        desc.read_exact(&mut bytes)?;
        if elf.little_endian {
            u32::from_le_bytes(bytes) as u64
        } else {
            u32::from_be_bytes(bytes) as u64
        }
    })
}
