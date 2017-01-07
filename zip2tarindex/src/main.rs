// src/main.rs -- the zip2tarindex conversion helper program
// Copyright 2017 The Tectonic Project
// Licensed under the MIT License.

extern crate clap;
extern crate tar;
extern crate zip;

use clap::{Arg, App};
use std::{fmt, path, process};
use std::fs::File;
use std::io::{stderr, Error, ErrorKind, Read, Result, Seek, SeekFrom, Write};
use std::ops::{Deref, DerefMut};
use zip::ZipArchive;

// Here is stuff from tar-rs's lib.rs

extern crate libc;
extern crate filetime;
//#[cfg(all(unix, feature = "xattr"))]
//extern crate xattr;

pub use header::{Header, OldHeader, UstarHeader, GnuHeader, GnuSparseHeader};
pub use header::{GnuExtSparseHeader};
pub use entry_type::EntryType;
//pub use entry::Entry;
//pub use archive::{Archive, Entries};
pub use builder::Builder;
pub use pax::{PaxExtensions, PaxExtension};

//mod archive;
mod builder;
//mod entry;
mod entry_type;
mod error;
mod header;
mod pax;

// FIXME(rust-lang/rust#26403):
//      Right now there's a bug when a DST struct's last field has more
//      alignment than the rest of a structure, causing invalid pointers to be
//      created when it's casted around at runtime. To work around this we force
//      our DST struct to instead have a forcibly higher alignment via a
//      synthesized u64 (hopefully the largest alignment we'll run into in
//      practice), and this should hopefully ensure that the pointers all work
//      out.
struct AlignHigher<R: ?Sized>(u64, R);

impl<R: ?Sized> Deref for AlignHigher<R> {
    type Target = R;
    fn deref(&self) -> &R { &self.1 }
}
impl<R: ?Sized> DerefMut for AlignHigher<R> {
    fn deref_mut(&mut self) -> &mut R { &mut self.1 }
}

fn other(msg: &str) -> Error {
    Error::new(ErrorKind::Other, msg)
}

// End tar-rs copy-paste.

fn die(args: fmt::Arguments) -> ! {
    writeln!(&mut stderr(), "error: {}", args).expect("write to stderr failed");
    process::exit(1)
}


fn main() {
    let matches = App::new("zip2tarindex")
        .version("0.1")
        .about("Convert a Zip file to a tar file with an index.")
        .arg(Arg::with_name("ZIPFILE")
             .help("The input Zip file to process.")
             .required(true)
             .index(1))
        .arg(Arg::with_name("TARPATH")
             .help("The name of the output tar file to create.")
             .required(true)
             .index(2))
        .get_matches();

    let zippath = matches.value_of("ZIPFILE").unwrap();
    let tarpath = matches.value_of("TARPATH").unwrap();

    // Open files.

    let mut zipfile = match File::open(zippath) {
        Ok(f) => f,
        Err(e) => die(format_args!("failed to open \"{}\": {}", zippath, e)),
    };

    let mut tarfile = match File::create(tarpath) {
        Ok(f) => f,
        Err(e) => die(format_args!("failed to create \"{}\": {}", tarpath, e)),
    };

    let mut indexpath = path::PathBuf::from(tarpath);
    let mut tar_fn = indexpath.file_name().unwrap().to_os_string();
    tar_fn.push(".index");
    indexpath.set_file_name(tar_fn);
    let mut indexfile = match File::create(&indexpath) {
        Ok(f) => f,
        Err(e) => die(format_args!("failed to create \"{}\": {}", indexpath.display(), e)),
    };

    // Let's go!

    let mut zip = match ZipArchive::new(zipfile) {
        Ok(a) => a,
        Err(e) => die(format_args!("couldn\'t open {} as a Zip file: {}", zippath, e))
    };

    let mut tar = tar::Builder::new(&mut tarfile);

    for i in 0..zip.len() {
        let mut file = zip.by_index(i).unwrap();
        let size = file.size();

        let mut buf = Vec::with_capacity(size as usize);
        file.read_to_end(&mut buf);
    }
}
