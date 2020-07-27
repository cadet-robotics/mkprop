extern(C) void test() nothrow @nogc {
    return;
}

struct ConstantPool {
    ushort prevId;
    ubyte[] data;
}

// Since field names must be ASCII (no \x00), we can
// just copy buffers
ushort append_utf8(ConstantPool *pool, string s) {
    if (pool.prevId == 0xfffe) {
        throw new Exception("too many constants");
    } else if (s.length > 0xffff) {
        throw new Exception("utf8 data too long");
    }
    ubyte[3] len_bytes;
    len_bytes[0] = 1;
    len_bytes[1..$] = convertToBe(cast(ushort) s.length);
    pool.data = pool.data ~ len_bytes ~ (cast(immutable(ubyte)[]) s);
    pool.prevId++;
    return pool.prevId;
}

ushort append_integer(ConstantPool *pool, int n) {
    if (pool.prevId == 0xfffe) {
        throw new Exception("too many constants");
    }
    pool.data = pool.data ~ [cast(ubyte) 3] ~ convertToBe(n);
    pool.prevId++;
    return pool.prevId;
}

ubyte[8] classSig(ushort majorVersion, ushort minorVersion) {
    ubyte[8] ret;
    ret[0..4] = [0xCA, 0xFE, 0xBA, 0xBE];
    ret[4..6] = convertToBe(minorVersion);
    ret[6..8] = convertToBe(majorVersion);
    return ret;
}

ubyte[T.sizeof] convertToBe(T)(T val) {
    ubyte[T.sizeof] ret;
    static foreach (i; 0..T.sizeof) {{
        enum shift = (T.sizeof - i - 1) * 8;
        ret[i] = (val >>> shift) & 0xff;
    }}
    return ret;
}

template Builder(string formatStr)