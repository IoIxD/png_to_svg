## PNGToSVG

A program for converting PNGs (and JPEGs) to SVGs without any tracing support or the like. A rectangle element is made for each unique set of pixels, and commonly used pixels are stored in defs to save on file size. If you are looking to convert a PNG to an SVG without sharp edges, you should be [here](https://www.pngtosvg.com/) instead.

The reason this exists is because on modern browsers, turning off anti-aliasing on images and hoping it actually works, let alone looks good, is a mess, and I would not be surprised if Chromium drops support for even doing it in the future. The best solution I have found is to play by their rules; by converting PNGs, pixel by pixel, to SVGs, we can force the browser to draw the pixels as we want them to, without any blurring or bleeding edges (the latter of which is particularly common on screens with uncommon DPIs).

This is an inheriently bad solution due to the file size (3 times bigger then a PNG ([though this could be improved.](https://github.com/IoIxD/PNGToSVG/issues/1))) but if you are looking to make a modern website that the general public might see, with pixel perfect/retro graphics, this is basically your only option, because on displays higher then 1080p (which are becoming more common) these kinds of images scale weirdly. It's slight, but it's enough to look awful, and this is where this solution comes in handy.

# Usage

`./PNGToSVG <one or more files>`

# Compiling

`cd ~/Projects/PNGToSVG`

`cargo run <one or more files>` or `cargo build --release`

i fucking love rust

to compile to all the supported platforms

```
cargo build --release --target=i686-unknown-linux-gnu
cargo build --release --target=x86_64-unknown-linux-gnu
cargo build --release --target=i686-pc-windows-gnu
cargo build --release --target=x86_64-pc-windows-gnu
```