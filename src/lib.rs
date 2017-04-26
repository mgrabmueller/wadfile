extern crate byteorder;

use std::io::Read;
use std::io::Seek;
use std::fs::File;
use std::io;
use std::collections::HashMap;

use byteorder::{LittleEndian, ReadBytesExt};

#[derive(Debug)]
pub enum WadType {
    IWAD,
    PWAD
}

#[derive(Debug)]
pub struct Lump {
    file_offset: i32,
    size: i32
}

#[derive(Debug)]
pub struct Header {
    pub wad_type: WadType,
    pub directory_entry_count: i32,
    pub directory_start: i32,
    pub directory: HashMap<String, Lump>
}

fn mk_err(msg: &str) -> io::Error {
    io::Error::new(io::ErrorKind::Other, msg)
}

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
                for i in c..8 {
                    if name[i] != 0 {
                        return Err(mk_err("non-0 after 0 character in lump name"))
                    }
                }
                break;
            }
            _ =>
                return Err(mk_err("invalid character in lump name"))
        }
    };
    Ok(())
}

/// Read header and directory information from the given WAD file.
/// Return an error indicator on general WAD file format errors.  The
/// contents of lumps is not checked, only the consistency of the
/// header and directory contents.
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

    let mut directory = HashMap::new();
    let mut lump_name = [0u8; 8];
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
        directory.insert(name, Lump{file_offset: lump_ptr, size: lump_size});
    }
    
    let hdr = Header{
        wad_type: wad_type,
        directory_entry_count: directory_entry_count,
        directory_start: directory_start,
        directory: directory
    };
    Ok(hdr)
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
