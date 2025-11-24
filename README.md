# lifpdf

Library for converting speed skating RaceResult `.lif` files to PDFs.

# Usage Instructions

You can enter directory paths in the settings menu. These will be watched, and files in these directories will be listed
in-app, sorted by last modified. After selecting one of these, a preview of the data will pop up. If it looks good,
click `Generate PDF` to generate the file. If that preview looks good, the `Print` button will bring up that document
using the native dialog. Also in settings, there is an option to automatically output the PDF files to a specified
directory whenever `Generate PDF` is clicked.

# Build Instructions

The Rust toolchain is required, and it can be downloaded [here](https://rust-lang.org/).

To build, run `cargo build --release` in the directory.

This application uses `fltk` to handle printing, and thus build dependencies for `fltk` may be required. If the build is
throwing linker errors, this is where to start. Which build dependencies are required for various systems can be
found [here](https://github.com/fltk-rs/fltk-rs?tab=readme-ov-file#build-dependencies).