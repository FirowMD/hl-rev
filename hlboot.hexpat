#pragma array_limit 4294967295
#pragma pattern_limit 0xFFFFFFFF

fn read_byte(u64 address) {
    u8 value @ address;
    return value;
};

struct HlbcVar {
    u8 b = read_byte(addressof(this)) [[hidden]];
    u32 value = 0 [[export]];
    if ((b & 0x80) == 0) {
        value = b & 0x7F;
        u8 seq[1];
    } else if ((b & 0x40) == 0) {
        u8 c = read_byte(addressof(this) + 1);
        value = c | ((b & 0x1F) << 8);
        u8 seq[2];
    } else {
        u8 c = read_byte(addressof(this) + 1);
        u8 d = read_byte(addressof(this) + 2);
        u8 e = read_byte(addressof(this) + 3);
        value = ((b & 0x1F) << 24) | (c << 16) | (d << 8) | e;
        u8 seq[4];
    }
};

struct HlbcRefString {
    HlbcVar data;
};

struct HlbcRefGlobal {
    HlbcVar data;
};

struct HlbcRefType {
    HlbcVar data;
};

struct HlbcRefFun {
    HlbcVar data;
};

struct HlbcRefField {
    HlbcVar data;
};

struct HlbcObjField {
    HlbcRefString name;
    HlbcRefType t;
};

struct HlbcObjProto {
    HlbcRefString name;
    HlbcRefFun findex;
    HlbcVar pindex;
};

struct HlbcHashMap {
    HlbcRefField key;
    HlbcRefFun value;
};

struct HlbcConstruct {
    HlbcRefString name;
    HlbcVar nparams;
    if (nparams.value > 0) {
        HlbcRefType params[nparams.value];
    }
};

struct HlbcTypeFun {
    u8 nargs;
    if (nargs > 0) {
        HlbcRefType args[nargs];
    }
    HlbcRefType ret;
};

struct HlbcTypeObj {
    HlbcRefString name;
    HlbcVar super_;
    HlbcRefGlobal global;
    HlbcVar nfields;
    HlbcVar nprotos;
    HlbcVar nbindings;

    if (nfields.value > 0) {
        HlbcObjField own_fields[nfields.value];
    }
    if (nprotos.value > 0) {
        HlbcObjProto protos[nprotos.value];
    }
    if (nbindings.value > 0) {
        HlbcHashMap bindings[nbindings.value];
    }
};

struct HlbcType {
    u8 byte;
    if (byte == 10) {
        HlbcTypeFun typefun;
    } else if (byte == 11) {
        HlbcTypeObj typeobj;
    } else if (byte == 14) {
        HlbcRefType reftype;
    } else if (byte == 15) {
        HlbcVar nfields;
        if (nfields.value > 0) {
            HlbcObjField fields[nfields.value];
        }
    } else if (byte == 17) {
        HlbcRefString name;
    } else if (byte == 18) {
        HlbcRefString name;
        HlbcRefGlobal global;
        HlbcVar nconstructs;
        if (nconstructs.value > 0) {
            HlbcConstruct constructs[nconstructs.value];
        }
    } else if (byte == 19) {
        HlbcRefType reftype;
    } else if (byte == 20) {
        HlbcTypeFun typefun;
    } else if (byte == 21) {
        HlbcTypeObj typeobj; 
    } else if (byte == 22) {
        HlbcRefType reftype;
    }
};

struct StrHlbc {
    char s[];
};

struct Hlbc {
    char magic[3];
    s8 version;
    HlbcVar flags;
    HlbcVar nints;
    HlbcVar nfloats;
    HlbcVar nstrings;
    if (version >= 5) {
        HlbcVar nbytes;
    }
    HlbcVar ntypes;
    HlbcVar nglobals;
    HlbcVar nnatives;
    HlbcVar nfunctions;
    if (version >= 4) {
        HlbcVar nconstants;
    }
    HlbcVar entrypoint;
    s32 ints[nints.value];
    double floats[nfloats.value];
    u32 string_data;
    char strings[string_data];
    HlbcVar string_sizes[nstrings.value];
    if (version >= 5) {
        u32 byte_data;
        u8 bytes[byte_data];
        HlbcVar byte_sizes[nbytes.value];
    }
    if (flags.value & 1 == 1) {
        HlbcVar ndebug_files;
        u32 file_data;
        char files[file_data];
        HlbcVar file_sizes[ndebug_files.value];
    }
    HlbcType types[ntypes.value];
};

Hlbc hlbc @ 0x00;