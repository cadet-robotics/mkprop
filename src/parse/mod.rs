use std::collections::HashMap;

mod parse_inner;

pub struct TemplateFile {
    pub class_name: String,
    pub mappings: HashMap<String, (String, bool)>
}

impl TemplateFile {
    pub fn parse(s: &str) -> Self {
        let parser = parse_inner::TemplateFileParser::new();
        let (class_name, mappings_raw) = parser.parse(s).unwrap();
        let mut mappings: HashMap<String, (String, bool)> = HashMap::new();
        let mut reverse_mappings: HashMap<String, String> = HashMap::new();
        for ent in mappings_raw {
            if mappings.contains_key(&ent.0) {
                panic!(format!("[template] duplicate mapping {} => ({} and {})", ent.0.as_str(), mappings.get(&ent.0).unwrap().0.as_str(), ent.1.as_str()))
            } else if reverse_mappings.contains_key(&ent.1) {
                panic!(format!("[template] duplicate mapping ({} and {}) => {}", ent.0.as_str(), reverse_mappings.get(&ent.1).unwrap().as_str(), ent.1.as_str()))
            }
            mappings.insert(ent.0.clone(), (ent.1.clone(), ent.2));
            reverse_mappings.insert(ent.1, ent.0);
        }
        TemplateFile {
            class_name,
            mappings
        }
    }
}

pub struct DriverDataFile {
    pub inner: HashMap<String, i32>
}

impl DriverDataFile {
    pub fn parse(s: &str) -> Self {
        let parser = parse_inner::DriverDataFileParser::new();
        let out = parser.parse(s).unwrap();
        let mut inner = HashMap::new();
        for ent in out {
            if inner.contains_key(&ent.0) {
                panic!(format!("[driver data] duplicate entry {} is both {} and {}", ent.0.as_str(), inner.get(&ent.0).unwrap(), ent.1));
            }
            inner.insert(ent.0, ent.1);
        }
        DriverDataFile {
            inner
        }
    }
}