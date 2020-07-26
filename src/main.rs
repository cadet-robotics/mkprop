extern crate cesu8;
extern crate clap;

use std::io::Write;
use clap::{App, SubCommand, Arg, AppSettings};
use std::fs::{read_to_string, File};
use parse::{TemplateFile, DriverDataFile};
use std::collections::HashMap;

mod parse;
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

            let mut template = TemplateFile::parse(template_d.as_str());
            let driver_data = DriverDataFile::parse(driver_data_d.as_str());

            let mut fields = HashMap::new();
            for d_ent in driver_data.inner {
                if template.mappings.contains_key(&d_ent.0) {
                    fields.insert(template.mappings.remove(&d_ent.0).unwrap().0, d_ent.1);
                } else {
                    eprintln!("[WARN] [BUILD] Unused driver data label \"{}\"", d_ent.0.as_str());
                }
            }

            for t_ent in template.mappings {
                if !(t_ent.1).1.clone() {
                    eprintln!("[WARN] [BUILD] Unused non-opt template \"{}\" -> \"{}\"", t_ent.0.as_str(), (t_ent.1).0.as_str());
                }
                fields.insert((t_ent.1).0.clone(), -1);
            }

            let mut out_file = File::create(out_file_f).unwrap();
            gen_class_file::gen_class_file(DEFAULT_CLASS_VERSION, template.class_name.as_str(), &fields, &mut out_file).unwrap();
            out_file.flush().unwrap();
        },
        ("sketch", Some(sm)) => {
            let template_f = sm.value_of("TEMPLATE").unwrap();
            let out_file_f = sm.value_of("OUT_FILE").unwrap();

            let template_d = read_to_string(template_f).unwrap();

            let template = TemplateFile::parse(template_d.as_str());

            let mut fields = HashMap::new();
            for ent in template.mappings {
                fields.insert((ent.1).0.clone(), -1);
            }

            let mut out_file = File::create(out_file_f).unwrap();
            gen_class_file::gen_class_file(DEFAULT_CLASS_VERSION, template.class_name.as_str(), &fields, &mut out_file).unwrap();
            out_file.flush().unwrap();
        },
        _ => unreachable!()
    }
}