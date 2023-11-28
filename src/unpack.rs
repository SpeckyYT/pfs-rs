use std::{path::Path, fs::{self, File}, io::{BufReader, Read, Seek, SeekFrom}};
use sha1::{Sha1, Digest};
use rayon::prelude::*;

use crate::artemis::{self, HEADER_SIZE};

pub fn unpack<P: AsRef<Path> + Sync>(file_path: P, folder: P) {
    let file_handle = File::open(&file_path).unwrap();
    let mut buf = BufReader::new(file_handle);

    let mut header_bytes = [ 0; artemis::HEADER_SIZE ];
    buf.read_exact(&mut header_bytes).expect("file not big enough");
    let header = artemis::Header::from_bytes(&header_bytes);

    if !header.has_magic() {
        panic!("invalid file (magic bytes do not match)")
    }

    match header.pack_version as char {
        '2'|'6'|'8' => println!("Valid PFS Version {} archive", header.pack_version as char),
        _ => panic!("unknown PFS version."),
    }

    println!("Index size: {a:#06x?} ({a})", a = header.index_size);
    println!("File count: {a:#06x?} ({a})", a = header.file_count);

    let mut files: Vec<artemis::Entry> = Vec::with_capacity(header.file_count.try_into().unwrap());

    for _ in 0..header.file_count {
        let mut filename_length = [0; 4];
        buf.read_exact(&mut filename_length).unwrap();
        let filename_length = u32::from_le_bytes(filename_length);
        
        let mut filename = vec![0; filename_length.try_into().unwrap()];
        buf.read_exact(&mut filename).unwrap();
        let filename = String::from_utf8(filename).unwrap();

        buf.read_exact(&mut [0; 4]).unwrap();

        let mut offset = [0; 4];
        buf.read_exact(&mut offset).unwrap();
        let offset = u32::from_le_bytes(offset);

        let mut size = [0; 4];
        buf.read_exact(&mut size).unwrap();
        let size = u32::from_le_bytes(size);

        files.push(artemis::Entry {
            path: filename,
            offset,
            size,
        });
    }

    let xor_key = match header.pack_version {
        b'8'.. => {
            buf.seek(SeekFrom::Start(HEADER_SIZE as u64 - std::mem::size_of::<u32>() as u64)).unwrap();
            let mut header_buf = vec![0; header.index_size.try_into().unwrap()];
            buf.read_exact(&mut header_buf).unwrap();
    
            let mut hasher = Sha1::new();
            hasher.update(header_buf);

            Some(hasher.finalize().into())
        },
        _ => None,
    };

    println!("XOR key: {:?}", xor_key);
    
    if header.pack_version == b'8' {
        
    }

    // let full_file = fs::read(file_path).unwrap();

    files.into_iter()
    .map(|entry| {
        buf.seek(SeekFrom::Start(entry.offset.try_into().unwrap())).unwrap();
        let mut bytes = vec![0; entry.size.try_into().unwrap()];
        buf.read_exact(&mut bytes).unwrap();

        (entry, bytes)
    })
    .par_bridge()
    .for_each(|(entry, mut bytes)| {
        if entry.offset == 0 {
            return;
        }

        let new_path = folder.as_ref().join(entry.path);
        let parent = new_path.parent().unwrap();

        if !parent.exists() {
            fs::create_dir_all(parent).unwrap();
        }

        println!("Extracting `{}`", new_path.file_name().unwrap().to_str().unwrap());

        if let Some(xor_key) = xor_key {
            crate::xor::xorcrypt(&mut bytes, &xor_key);
        }

        fs::write(new_path, bytes).unwrap();
    });
}
