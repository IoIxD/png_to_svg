extern crate derive_more;

use std::env;
use std::fmt::Write;
use std::io::{Error};
use regex::Regex;
use image::{GenericImageView};
use derive_more::IntoIterator;

fn main() {
    let mut svg_files = vec![String::from(""); 1];
    let files = get_files();

    if files.len() <= 0 {
        println!("No images, or images of valid types, specified to convert.");
        return;
    }
    
    for file in files {
        let f = match svg_from_file(file) {
            Ok(a) => a,
            Err(err) => {
                println!("{}", err);
                continue;
            }
        };
        svg_files.push(f);
    }
    println!("{}",svg_files.concat());
}

#[derive(Clone)]
struct SVGDef {
    pub i: i32,
    pub contents: String, // todo: proper svg object 
}
impl SVGDef {
    fn to_string(&mut self) -> String {
        format!("<g id='{}'>{}</g>",&self.i, &self.contents.replace("\n", ""))
    }
    fn to_use_string(&mut self, x: u32, y: u32) -> String {
        format!("<use x='{}' y='{}' href='#{}'></use>",x,y,&self.i)
    }
}

#[derive(IntoIterator)]
struct SVGDefs {
    #[into_iterator(ref)]
    defs: Vec<SVGDef>,
}

impl SVGDefs {
    fn new() -> SVGDefs {
        SVGDefs{defs: Vec::<SVGDef>::new()}
    }
    fn contains(&self, c: &String) -> Option<SVGDef> {
        // todo: there's probably a good one liner i can do here
        for def in &self.defs {
            if def.contents == *c {
                return Some(def.clone());
            }
        };
        return None;
    }
    fn add(&mut self, c: &String) {
        self.defs.push(SVGDef{
            i: self.defs.len() as i32,
            contents: c.clone(),
        });
    }
}

fn svg_from_file(file: String) -> Result<String, Error> {
    let img = match image::open(file) {
        Ok(a) => a,
        Err(err) => panic!("Unable to open file; {}",err),
    };

    let (width, height) = (img.width(),img.height());

    let mut svg_lines = vec![String::from(""); 0];
    let mut svg_defs = SVGDefs::new();

    // buffer for storing the last rgb value we got.
    let (mut last_r, mut last_g, mut last_b, mut last_a): (u16, u16, u16, u16)
     = (300, 300, 300, 300);
    let mut cur_width: u32 = 1;
    let (mut last_x, mut last_y): (u32, u32) = (0,0);

    let mut making_box: bool = false;

    for y in 0..height {
        for x in 0..width {
            let pixels = img.get_pixel(x, y);
            let (r, g, b, a) = (pixels[0] as u16, pixels[1] as u16, 
                                pixels[2] as u16, pixels[3] as u16);
            // don't process shit if we're on a transparent pixel
            // (unless that last pixel we checked wasn't transparento)
            if a > 0 || last_a > 0 {
                // if what we have is different from what is stored, 
                // or the width of what we have is larger then the image,
                // make a new box
                if r != last_r && g != last_g && b != last_b {
                    // first though, check if there's a definition that matches they box.
                    svg_lines.push(new_box(cur_width, last_x, last_y, last_r, last_g, last_b, &mut svg_defs));
                    making_box = false;
                    cur_width = 1;
                    (last_r, last_g, last_b, last_a) = (r, g, b, a);
                    (last_x, last_y) = (x,y);
                // otherwise, increase that width value that we have until we get a new box.
                } else {
                    making_box = true;
                    cur_width += 1;
                }
            };
        }
        // if we were making a box, make what we have and move on.
        if making_box {
            svg_lines.push(new_box(cur_width, last_x, last_y, last_r, last_g, last_b, &mut svg_defs));
            making_box = false;
            cur_width = 1;
            (last_r, last_g, last_b, last_a) = (300, 300, 300, 300);
            (last_x, last_y) = (0,y);
        }
    };

    // buffer for the svg code.
    let mut string_buffer = String::from(
        format!("<svg viewBox='0 0 {} {}' xmlns='http://www.w3.org/2000/svg'>",width,height)
    );

    _ = string_buffer.write_str("<defs>");
    for mut def in svg_defs {
        _ = string_buffer.write_str(def.to_string().as_str());
    };
    _ = string_buffer.write_str("</defs>");
    for line in svg_lines {
        _ = string_buffer.write_str(&line);
    };
    _ = string_buffer.write_str("</svg>");

    Ok(string_buffer)
}

fn get_files() -> Vec<String> {
    let valid_types = Regex::new(r"\.(png|jpeg|jpg)$").unwrap();

    env::args().into_iter().filter(|i| {
        valid_types.find(i).is_some()
    }).collect()
}

fn new_box(width: u32, x: u32, y: u32, r: u16, g: u16, b: u16, svg_defs: &mut SVGDefs) -> String {
    if r+g+b >= 255*3 {
        format!("")
    } else {
        let line = new_box_without_pos(width, r, g, b);
        match svg_defs.contains(&line) {
            // if there is a line...
            Some(mut a) => {
                a.to_use_string(x,y)
            },
            // if there isn't.
            None => {
                svg_defs.add(&line);
                format!("<rect width='{}' height='{}' x='{}' y='{}' fill='{}'></rect>",
            width as f32+0.2,1.2,x,y,rgb_to_hex(r,g,b))
            }
        }
    }
    
}

fn new_box_without_pos(width: u32, r: u16, g: u16, b: u16) -> String {
    if r+g+b >= 255*3 {
        format!("")
    } else {
        format!("<rect width='{}' height='{}' fill='{}'></rect>",
            width as f32+0.2,1.2,rgb_to_hex(r,g,b))
    }
}

#[inline(always)]
fn rgb_to_hex(r: u16, g: u16, b: u16) -> String {
    format!("#{:02X}{:02X}{:02X}",r, g, b)
}
// there is an EXTREMELY weird bug where the "if(a > 1)" rings true immediately after we enter a transparency area (even though it's now 0), and I cannot for the life of me find out what it is. maybe one day i will, but it seems the best solution is actually to prevent the program from making any black boxes, and PNGs will need to be modified to use extremely dark greys (254,254,254) instead.
