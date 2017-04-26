// Copyright 2016 Martin Grabmueller. See the LICENSE file at the
// top-level directory of this distribution for license information.

extern crate wadfile;

pub fn main() {
    for f in std::env::args().skip(1) {
        println!("WAD file: {}", f);
        match wadfile::read_header(&f) {
            Ok(hdr) => {
                println!("  WAD type: {:?}", hdr.wad_type);
                println!("  # of lumps: {}", hdr.directory_entry_count);
                println!("  directory start: {}", hdr.directory_start);
            },
            Err(err) => {
                println!("  Could not read WAD: {}", err);
            }
        }
    }
}

