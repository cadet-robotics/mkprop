use nom::lib::std::collections::HashMap;
use std::io::Write;
use cesu8::to_java_cesu8;

struct ConstPool {
    data: Vec<u8>,
    prev_id: u16
}

const CONST_SELF_ID: u16 = 2;
const CONST_I_ID: u16 = 3;
const CONST_VALUE_ID: u16 = 4;
const CONST_SUPER: u16 = 6;
const CONST_INIT: u16 = 7;
const CONST_INIT_DESC: u16 = 8;
const CONST_CODE_ID: u16 = 9;
const CONST_SINIT_REF: u16 = 11;

impl ConstPool {
    fn new() -> Self {
        ConstPool {
            data: Vec::new(),
            prev_id: 11
        }
    }

    fn insert_utf8(&mut self, s: &str) -> u16 {
        if self.prev_id == 0xfffe {
            panic!("too many constants")
        }

        let d = to_java_cesu8(s);
        let len = d.len();
        if len > 0xffff {
            panic!("utf8 data too long")
        }
        let len = len as u16;

        self.data.extend_from_slice(&[
            1,
            (len >> 8) as u8,
            (len & 0xff) as u8
        ]);
        self.data.extend_from_slice(d.as_ref());

        self.prev_id += 1;
        self.prev_id
    }

    fn insert_integer(&mut self, n: i32) -> u16 {
        if self.prev_id == 0xfffe {
            panic!("too many constants")
        }

        self.data.extend_from_slice(&[
            3,
            (n >> 24) as u8,
            ((n >> 16) & 0xff) as u8,
            ((n >> 8) & 0xff) as u8,
            (n & 0xff) as u8
        ]);

        self.prev_id += 1;
        self.prev_id
    }

    fn write_out(self, class_name: &str, writer: &mut impl Write) -> std::io::Result<()> {
        let len = self.prev_id + 1;
        writer.write_all(&[
            (len >> 8) as u8,
            (len & 0xff) as u8
        ])?;
        // Writes special constants first
        write_utf8_const(class_name, writer)?;
        writer.write_all(&[
            // 2
            7,
            0,
            1,
            // 3
            1,
            0,
            1,
            b'I',
            // 4
            1,
            0,
            13,
            b'C',
            b'o',
            b'n',
            b's',
            b't',
            b'a',
            b'n',
            b't',
            b'V',
            b'a',
            b'l',
            b'u',
            b'e',
            // 5
            1,
            0,
            16,
            b'j',
            b'a',
            b'v',
            b'a',
            b'/',
            b'l',
            b'a',
            b'n',
            b'g',
            b'/',
            b'O',
            b'b',
            b'j',
            b'e',
            b'c',
            b't',
            // 6
            7,
            0,
            5,
            // 7
            1,
            0,
            6,
            b'<',
            b'i',
            b'n',
            b'i',
            b't',
            b'>',
            // 8
            1,
            0,
            3,
            b'(',
            b')',
            b'V',
            // 9
            1,
            0,
            4,
            b'C',
            b'o',
            b'd',
            b'e',
            // 10
            12,
            0,
            7,
            0,
            8,
            // 11
            10,
            0,
            6,
            0,
            10
        ])?;
        // Writes non special
        writer.write_all(self.data.as_slice())
    }
}

struct Fields {
    data: Vec<u8>,
    len: u16
}

impl Fields {
    fn new() -> Self {
        Fields {
            data: Vec::new(),
            len: 0
        }
    }

    fn insert_int_field(&mut self, pool: &mut ConstPool, k: &str, v: i32) {
        if self.len == 0xffff {
            panic!("too many fields")
        }

        let k_id = pool.insert_utf8(k);
        let v_id = pool.insert_integer(v);
        const FLAGS: u16 = 0x1 | 0x8 | 0x10 | 0x1000;

        self.data.extend_from_slice(&[
            (FLAGS >> 8) as u8,
            (FLAGS & 0xff) as u8,
            (k_id >> 8) as u8,
            (k_id & 0xff) as u8,
            (CONST_I_ID >> 8) as u8,
            (CONST_I_ID & 0xff) as u8,
            0,
            1,
            (CONST_VALUE_ID >> 8) as u8,
            (CONST_VALUE_ID & 0xff) as u8,
            0,
            0,
            0,
            2,
            (v_id >> 8) as u8,
            (v_id & 0xff) as u8
        ]);

        self.len += 1;
    }

    fn write_out(self, writer: &mut impl Write) -> std::io::Result<()> {
        writer.write_all(&[
            (self.len >> 8) as u8,
            (self.len & 0xff) as u8
        ])?;
        writer.write_all(self.data.as_slice())
    }
}

// Write CONSTANT_Utf8_info
fn write_utf8_const(s: &str, writer: &mut impl Write) -> std::io::Result<()> {
    let d = to_java_cesu8(s);
    let len = d.len();
    if len > 0xffff {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "utf8 data too long"))
    }
    let len = len as u16;
    writer.write_all(&[
        1,
        (len >> 8) as u8,
        (len & 0xff) as u8
    ])?;
    writer.write_all(d.as_ref())
}

pub(crate) fn gen_class_file(version: (u16, u16), class_name: &str, map: &HashMap<String, i32>, writer: &mut impl Write) -> Result<(), std::io::Error> {
    // Write magic and version
    writer.write_all(&[
        0xca, 0xfe, 0xba, 0xbe,
        (version.1 >> 8) as u8,
        (version.1 & 0xff) as u8,
        (version.0 >> 8) as u8,
        (version.0 & 0xff) as u8
    ])?;

    // Build fields and constants
    let mut c_pool = ConstPool::new();
    let mut f_pool = Fields::new();
    for ent in map.iter() {
        f_pool.insert_int_field(&mut c_pool, ent.0.as_str(), *ent.1)
    }

    // Write const pool
    c_pool.write_out(class_name, writer)?;

    // Write access flags, this_class, super_class, empty interfaces[]
    const ACCESS_FLAGS: u16 = 0x1 | 0x10 | 0x20 | 0x1000;
    writer.write_all(&[
        (ACCESS_FLAGS >> 8) as u8,
        (ACCESS_FLAGS & 0xff) as u8,
        (CONST_SELF_ID >> 8) as u8,
        (CONST_SELF_ID & 0xff) as u8,
        (CONST_SUPER >> 8) as u8,
        (CONST_SUPER & 0xff) as u8,
        0,
        0
    ])?;

    // Write fields
    f_pool.write_out(writer)?;

    // Write methods and empty attributes[]
    const FLAGS_INIT: u16 = 0x2 | 0x1000;
    writer.write_all(&[
        0,
        1,
        (FLAGS_INIT >> 8) as u8,
        (FLAGS_INIT & 0xff) as u8,
        (CONST_INIT >> 8) as u8,
        (CONST_INIT & 0xff) as u8,
        (CONST_INIT_DESC >> 8) as u8,
        (CONST_INIT_DESC & 0xff) as u8,
        0,
        1,
        (CONST_CODE_ID >> 8) as u8,
        (CONST_CODE_ID & 0xff) as u8,
        0,
        0,
        0,
        17,
        0,
        1,
        0,
        1,
        0,
        0,
        0,
        5,
        0x2a, // aload_0
        0xb7, // invokespecial
        (CONST_SINIT_REF >> 8) as u8,
        (CONST_SINIT_REF & 0xff) as u8,
        0xb1, // return
        0,
        0,
        0,
        0,
        // No class attributes
        0,
        0
    ])
}