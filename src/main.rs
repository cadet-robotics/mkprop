extern crate nom;
extern crate cesu8;
extern crate clap;
extern crate bimap;

use nom::lib::std::collections::HashMap;
use std::io::Write;
use clap::{App, SubCommand, Arg, AppSettings};
use crate::parsing::{parse_template, parse_driver_data, Template, DefineStatement};
use std::fs::{read_to_string, File};
use bimap::BiHashMap;

mod parsing;
mod gen_class_file;

const DEFAULT_CLASS_VERSION: (u16, u16) = (52, 0);

fn main() {
    let app = App::new("mkprop")
        .about("A robotics code config generator")
        .subcommand(
            SubCommand::with_name("build")
                .about("creates a class file from a template and driver data")
                .arg(
                    Arg::with_name("TEMPLATE")
                        .help("the template file")
                        .required(true)
                        .index(1)
                )
                .arg(
                    Arg::with_name("DRIVER_DATA")
                        .help("the driver data file")
                        .required(true)
                        .index(2)
                )
                .arg(
                    Arg::with_name("OUT_FILE")
                        .help("the file to write class data to")
                        .required(true)
                        .index(3)
                )
        )
        .subcommand(
            SubCommand::with_name("sketch")
                .about("creates a class file from a template, setting all values to -1")
                .arg(
                    Arg::with_name("TEMPLATE")
                        .help("the template file")
                        .required(true)
                        .index(1)
                )
                .arg(
                    Arg::with_name("OUT_FILE")
                        .help("the file to write class data to")
                        .required(true)
                        .index(2)
                )
        )
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .get_matches();

    match app.subcommand() {
        ("build", Some(sm)) => {
            let template_f = sm.value_of("TEMPLATE").unwrap();
            let driver_data_f = sm.value_of("DRIVER_DATA").unwrap();
            let out_file_f = sm.value_of("OUT_FILE").unwrap();

            let template_d = read_to_string(template_f).unwrap();
            let driver_data_d = read_to_string(driver_data_f).unwrap();

            let template = parse_template(template_d.as_str()).unwrap();
            let driver_data = parse_driver_data(driver_data_d.as_str()).unwrap();

            let mut template = ProcessedTemplate::process(&template);
            let driver_data = ProcessedDriverData::process(&driver_data);

            let mut fields = HashMap::new();
            for d_ent in driver_data.data.iter() {
                if template.replacements.contains_left(d_ent.0) {
                    fields.insert(template.replacements.remove_by_left(d_ent.0).unwrap().1, *d_ent.1);
                } else {
                    eprintln!("[WARN] [BUILD] Unused driver data label \"{}\"", d_ent.0.as_str());
                }
            }

            for t_ent in template.replacements.iter() {
                if !template.is_const_opt.get(t_ent.1.as_str()).unwrap() {
                    eprintln!("[WARN] [BUILD] Unused non-opt template \"{}\" -> \"{}\"", t_ent.0.as_str(), t_ent.1.as_str());
                }
            }

            let mut out_file = File::create(out_file_f).unwrap();
            gen_class_file::gen_class_file(DEFAULT_CLASS_VERSION, template.class_name.as_str(), &fields, &mut out_file).unwrap();
            out_file.flush().unwrap();
        },
        ("sketch", Some(sm)) => {
            let template_f = sm.value_of("TEMPLATE").unwrap();
            let out_file_f = sm.value_of("OUT_FILE").unwrap();

            let template_d = read_to_string(template_f).unwrap();

            let template = parse_template(template_d.as_str()).unwrap();

            let template = ProcessedTemplate::process(&template);

            let mut fields = HashMap::new();
            for ent in template.replacements.iter() {
                fields.insert(ent.1.clone(), -1);
            }

            let mut out_file = File::create(out_file_f).unwrap();
            gen_class_file::gen_class_file(DEFAULT_CLASS_VERSION, template.class_name.as_str(), &fields, &mut out_file).unwrap();
            out_file.flush().unwrap();
        },
        _ => unreachable!()
    }
}

struct ProcessedTemplate {
    class_name: String,
    replacements: BiHashMap<String, String>,
    is_const_opt: HashMap<String, bool>
}

impl ProcessedTemplate {
    fn process(t: &Template) -> Self {
        let class_name = String::from(t.class_name);
        let mut replacements = BiHashMap::new();
        let mut is_const_opt = HashMap::new();
        for ent in t.maps.iter() {
            let driver_data = String::from(ent.driver_data_name);
            let const_name = String::from(ent.const_name);
            if replacements.contains_left(&driver_data) {
                eprintln!("[WARN] [TEMPLATE] Duplicate (x -> y, x -> z) where x = {}", ent.driver_data_name);
            } else if replacements.contains_right(&const_name) {
                eprintln!("[WARN] [TEMPLATE] Duplicate (x -> y, z -> y) where y = {}", ent.const_name);
            }
            replacements.insert(driver_data, const_name.clone());
            is_const_opt.insert(const_name, ent.opt);
        }
        ProcessedTemplate {
            class_name,
            replacements,
            is_const_opt
        }
    }
}

struct ProcessedDriverData {
    data: HashMap<String, i32>
}

impl ProcessedDriverData {
    fn process(dd: &Vec<DefineStatement>) -> Self {
        let mut data = HashMap::new();
        for ent in dd {
            if data.contains_key(ent.name) {
                eprintln!("[WARN] [DRIVER DATA] Duplicate (x -> [y], x -> [z]) where x = {}", ent.name);
            }
            data.insert(String::from(ent.name), ent.id);
        }
        ProcessedDriverData {
            data
        }
    }
}