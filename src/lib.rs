// Copyright 2016 Martin Grabmueller. See the LICENSE file at the
// top-level directory of this distribution for license information.

extern crate byteorder;

use std::io::Read;
use std::io::Seek;
use std::fs::File;
use std::io;

use byteorder::{LittleEndian, ReadBytesExt};

/// WAD files come in two flavours: IWAD and PWAD.
#[derive(Debug)]
pub enum WadType {
    /// IWADs are the main game files.  Running a game always requires
    /// one IWAD.
    IWAD,
    /// PWADs, or "Patch WADs" can override most of the lumps in an
    /// IWAD.  PWADs are loaded in addition to an IWAD file.
    PWAD
}

/// Individual data items are stored in lumps, which are named binary
/// blobs.
#[derive(Debug)]
pub struct Lump {
    /// Byte offset in the WAD file where the lump data starts.
    pub file_offset: i32,
    /// Length of the lump in bytes.
    pub size: i32
}

/// The `Header` structure contains information about the WAD file
/// type and the WAD file directory.
#[derive(Debug)]
pub struct Header {
    /// Type of the WAD.
    pub wad_type: WadType,
    /// Number of lumps in the WAD file.
    pub directory_entry_count: i32,
    /// Byte offset in the WAD file where the lump directory starts.
    pub directory_start: i32,
    /// The lump names of the WAD file in the same order as in the
    /// directory.
    pub lumps: Vec<(String, Lump)>,
}

/// Helper to create io::Error values.
fn mk_err(msg: &str) -> io::Error {
    io::Error::new(io::ErrorKind::Other, msg)
}

/// Check the validity of lump names.
fn validate_lump_name(name: &[u8; 8]) -> Result<(), io::Error> {
    for c in 0..8 {
        match name[c] {
            b'A'...b'Z' => {},
            b'0'...b'9' => {},
            b'[' => {},
            b']' => {},
            b'-' => {},
            b'_' => {},
            b'\\' => {},
            0 => {
                if c == 0 {
                    return Err(mk_err(&format!("{:?}: empty lump name", String::from_utf8_lossy(name))))
                }
                for i in c..8 {
                    if name[i] != 0 {
                        return Err(mk_err(&format!("{:?}: non-0 after 0 character in lump name", String::from_utf8_lossy(name))))
                    }
                }
                break;
            }
            ch =>
                return Err(mk_err(&format!("{:?}: invalid character in lump name: {:?}", String::from_utf8_lossy(name), ch as char)))
        }
    };
    Ok(())
}

/// Read header and directory information from the given WAD file.
///
/// # Errors
///
/// IO errors are directly returned.  When an inconsistency is
/// detected, an error of kind `ErrorKind:Other` is returned.  Note
/// that the lumps in the WAD are not checked, only the header and
/// lump directory.
pub fn read_header(wad_filename: &str) -> Result<Header, io::Error> {
    let mut f = try!(File::open(wad_filename));
    let metadata = try!(f.metadata());
    let file_size = metadata.len();
    
    let mut magic = [0u8; 4];

    try!(f.read_exact(&mut magic));

    let wad_type =
        if &magic[..] == b"IWAD" {
            WadType::IWAD
        } else if &magic[..] == b"PWAD" {
            WadType::PWAD
        } else {
            return Err(mk_err("invalid WAD tag"));
        };

    let directory_entry_count = try!(f.read_i32::<LittleEndian>());
    let directory_start = try!(f.read_i32::<LittleEndian>());

    if directory_entry_count < 0 {
        return Err(mk_err("directory entry count is negative"));
    }
    if directory_start < 0 {
        return Err(mk_err("directory start is negative"));
    }

    let dir_pos = try!(f.seek(io::SeekFrom::Start(directory_start as u64)));
    if dir_pos != directory_start as u64 {
        return Err(mk_err("cannot seek to directory start"));
    }

    let mut lump_name = [0u8; 8];
    let mut lump_names = Vec::new();

    for _ in 0..directory_entry_count {
        let lump_ptr = try!(f.read_i32::<LittleEndian>());
        if lump_ptr < 0 {
            return Err(mk_err("lump start pointer is negative"));
        } else if lump_ptr as u64 > file_size {
            return Err(mk_err("lump start pointer is too large"));
        }
        let lump_size = try!(f.read_i32::<LittleEndian>());
        if lump_size < 0 {
            return Err(mk_err("lump size is negative"));
        } else if lump_ptr as u64 + lump_size as u64 > file_size {
            return Err(mk_err("lump size is too large"));
        }
        try!(f.read_exact(&mut lump_name));

        try!(validate_lump_name(&lump_name));
        
        let mut d = 8;
        while d > 0 && lump_name[d - 1] == 0 {
            d -= 1;
        }
        let name = String::from(String::from_utf8_lossy(&lump_name[..d]));
        let entry = (name, Lump{file_offset: lump_ptr, size: lump_size});
        lump_names.push(entry);
    }
    
    let hdr = Header{
        wad_type: wad_type,
        directory_entry_count: directory_entry_count,
        directory_start: directory_start,
        lumps: lump_names,
    };
    Ok(hdr)
}

#[cfg(test)]
mod tests {
    use super::validate_lump_name;
    
    #[test]
    fn validate_lump_name0() {
        assert!(validate_lump_name(b"MAP32\0\0\0").is_ok());
    }
    #[test]
    fn validate_lump_name1() {
        assert!(validate_lump_name(b"A\0\0\0\0\0\0\0").is_ok());
    }
    #[test]
    fn validate_lump_name2() {
        assert!(validate_lump_name(b"ARCHVILE").is_ok());
    }
    #[test]
    fn validate_lump_name3() {
        assert!(validate_lump_name(b"\0\0\0\0\0\0\0\0").is_err());
    }
    #[test]
    fn validate_lump_name4() {
        assert!(validate_lump_name(b"\0X\0\0\0\0\0\0").is_err());
    }
    #[test]
    fn validate_lump_name5() {
        // This appears in Memento Mori's MM.WAD.
        assert!(validate_lump_name(b"DEMO3\0\0S").is_err());
    }
}
