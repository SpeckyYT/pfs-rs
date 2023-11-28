use std::{path::Path, fs, sync::{Arc, Mutex}};
use folder::scan;
use pathdiff::diff_paths;
use rayon::prelude::*;
use sha1::{Sha1, Digest};

use crate::artemis::{self, HEADER_SIZE};
use crate::xor::xorcrypt;

pub fn pack<P: AsRef<Path> + Sync>(dir: P, file: P) {
    let index_size = Arc::new(Mutex::new(0));
    let total_size = Arc::new(Mutex::new(HEADER_SIZE));

    let index = scan(
        &dir, 
        |path| !path.is_dir(),
        |path,_| Ok(fs::read(path).unwrap()),
        (),
        std::thread::available_parallelism().map(|p| p.get()).unwrap_or(1),
    )
    .par_bridge()
    .map(|(a,b)| (a,b.unwrap()))
    .map(|(file_path, content)| {
        let path = diff_paths(file_path, &dir).unwrap().to_str().unwrap().to_string();
        let size = content.len();

        let entry_size = 16 + path.len();
        println!("{}", entry_size);

        *index_size.lock().unwrap() += entry_size;
        *total_size.lock().unwrap() += entry_size + size;

        (artemis::Entry {
            path,
            size: size.try_into().unwrap(),
            ..Default::default()
        }, content)
    })
    .collect::<Vec<_>>();

    let index_size = *index_size.lock().unwrap();
    let total_size = *total_size.lock().unwrap();

    let header = artemis::Header {
        file_count: index.len().try_into().unwrap(),
        index_size: index_size.try_into().unwrap(),
        pack_version: b'8',
        ..Default::default()
    };

    let mut bytes: Vec<u8> = Vec::with_capacity(total_size);

    bytes.extend(&header.to_bytes());

    let mut offset: u32 = index_size.try_into().unwrap();

    for (entry, _) in &index {
        bytes.extend(<u32>::try_from(entry.path.len()).unwrap().to_le_bytes());
        bytes.extend(entry.path.as_bytes());
        bytes.extend([0; 4]);
        bytes.extend(offset.to_le_bytes());
        bytes.extend(entry.size.to_le_bytes());
        offset += entry.size;
    }

    bytes.extend(header.file_count.to_le_bytes());

    for (_, _) in &index {

    }

    let mut hasher = Sha1::new();
    hasher.update(&bytes[HEADER_SIZE-4..]);
    let xor_key: [u8; 20] = hasher.finalize().into();

    println!("XOR key: {:?}", xor_key);

    let all_files = index.into_par_iter().map(|(_, mut content)| {
        xorcrypt(&mut content, &xor_key);
        content
    })
    .flatten()
    .collect::<Vec<u8>>();

    bytes.extend(all_files);

    fs::write(file, bytes).unwrap();
    // let pfs_file: Vec<u8> = Vec::with_capacity(file_count * 1024);

    // println!("{:?}", size_hint);

    
}
