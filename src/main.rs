extern crate derive_more;

use std::env;
use std::fmt::Write;
use std::fs;
use std::io::{Error};
use lazy_static::lazy_static;
use regex::Regex;
use image::{GenericImageView};
use derive_more::IntoIterator;

lazy_static! {
    static ref VALID_TYPES: Regex = Regex::new(r"\.(png|jpeg|jpg)$").unwrap();
}

// TODO: proper svg package

fn main() {
    let files = get_files();

    if files.len() <= 0 {
        println!("No images, or images of valid types, specified to convert.");
        return;
    }
    
    for file in files {
        match svg_from_file(&file) {
            Ok(a) => {
                let newname = format!("{}.svg",VALID_TYPES.replace(&file, ""));
                println!("{} -> {}",&file, &newname);
                fs::write(newname, a)
                .expect(format!("failed to save file {}!",file).as_str());
            },
            Err(err) => {
                println!("{}", err);
                continue;
            }
        };
        
    }
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

fn svg_from_file(file: &String) -> Result<String, Error> {
    let img = match image::open(file) {
        Ok(a) => a,
        Err(err) => panic!("Unable to open file; {}",err),
    };

    let (width, height) = (img.width(),img.height());

    let mut svg_lines = vec![String::from(""); 0];
    let mut svg_defs = SVGDefs::new();

    // buffer for storing the last rgb value we got.
    let (mut last_r, mut last_g, mut last_b, mut last_a): (u8, u8, u8, u8) = (0,0,0,0);
    let mut cur_width: u32 = 1;
    let (mut last_x, mut last_y): (u32, u32) = (0,0);
    
    let mut compare_to_last: bool = false;

    for y in 0..height {
        for x in 0..width {
            let pixels = img.get_pixel(x, y);
            let (r, g, b, a) = (pixels[0], pixels[1],  pixels[2], pixels[3]);
            
            // don't do anything if we're on the first pixel.
            if !compare_to_last {
                (last_r, last_g, last_b, last_a) = (r, g, b, a);
                (last_x, last_y) = (x,y);
                compare_to_last = true;
                continue;
            }

            // don't process shit if we're on a transparent pixel
            // (unless that last pixel we checked wasn't transparent)
            if a > 0 || last_a > 0 {
                // if what we have is different from what is stored, 
                // or the width of what we have is larger then the image,
                // make a new box
                if r != last_r || g != last_g || b != last_b || a != last_a ||
                    cur_width+last_x >= width {
                    // first though, check if there's a definition that matches they box.
                    svg_lines.push(new_box(cur_width, last_x, last_y, last_r, last_g, last_b, Some(&mut svg_defs)));
                    cur_width = 1;
                    (last_r, last_g, last_b, last_a) = (r, g, b, a);
                    (last_x, last_y) = (x,y);
                // otherwise, increase that width value that we have until we get a new box.
                } else {
                    cur_width += 1;
                }
            };
        }
    };

    // buffer for the svg code.
    let mut string_buffer = String::from(
        format!("<svg viewBox='0 0 {} {}' xmlns='http://www.w3.org/2000/svg'>",width,height)
    );

    _ = string_buffer.write_str("<defs>");
    for mut def in svg_defs {
        // hack: singular black dots with an id of 0 shouldn't be added,
        // due to a bug I can't seem to find in transparent images.
        if def.i == 0 && def.contents.eq(&new_box_without_pos(1, 0, 0, 0)) {
            continue;
        } else {
            _ = string_buffer.write_str(def.to_string().as_str());
        }
        
    };
    _ = string_buffer.write_str("</defs>");
    let mut i = 0;
    for line in svg_lines {
        // same hack as described above
        if i == 0 && line.eq(&new_box(1, 0, 0, 0, 0, 0, None)) {
            continue;
        } else {
            _ = string_buffer.write_str(&line);
            i += 1;
        }
    };
    _ = string_buffer.write_str("</svg>");

    Ok(string_buffer)
}

fn get_files() -> Vec<String> {
    env::args().into_iter().filter(|i| {
        VALID_TYPES.find(i).is_some()
    }).collect()
}

fn new_box(width: u32, x: u32, y: u32, r: u8, g: u8, b: u8, svg_defs: Option<&mut SVGDefs>) -> String {
    let line = new_box_without_pos(width, r, g, b);
    
    match svg_defs {
        Some(a) => {
            match a.contains(&line) {
                // if there is a line...
                Some(mut a) => {
                    // hack: singular black dots with an id of 0 should be removed,
                    // due to a bug I can't seem to find in transparent images.
                    if a.i == 0 && width == 1 && r == 0 && g == 0 && b == 0 {
                        "".to_string()
                    } else {
                        a.to_use_string(x,y)
                    }
                    
                },
                // if there isn't.
                None => {
                    a.add(&line);
                    format!("<rect width='{}' height='{}' x='{}' y='{}' fill='{}'></rect>",
                width as f32+0.2,1.2,x,y,rgb_to_hex(r,g,b))
                }
            }
        }
        None => {
            format!("<rect width='{}' height='{}' x='{}' y='{}' fill='{}'></rect>",
                width as f32+0.2,1.2,x,y,rgb_to_hex(r,g,b))
        }
    }
    
}

fn new_box_without_pos(width: u32, r: u8, g: u8, b: u8) -> String {
    format!("<rect width='{}' height='{}' fill='{}'></rect>",
            width as f32+0.2,1.2,rgb_to_hex(r,g,b))
}

#[inline(always)]
fn rgb_to_hex(r: u8, g: u8, b: u8) -> String {
    format!("#{:02X}{:02X}{:02X}",r, g, b)
}