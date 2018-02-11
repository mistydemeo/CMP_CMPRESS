// build.rs

extern crate cc;

fn main() {
    cc::Build::new()
        .file("compress_rtns.c")
        .compile("compress_rtns");
}
