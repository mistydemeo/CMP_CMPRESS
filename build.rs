// Builds the C component of the crate

extern crate cc;

fn main() {
    cc::Build::new()
        .file("compress_rtns.c")
        .compile("compress_rtns");
}
