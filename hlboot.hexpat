#pragma array_limit 4294967295
#pragma pattern_limit 0xFFFFFFFF

import std.io;

u8 gHasDebug = 0;
u8 gVersion = 0;
u32 gTypeNumber = 0;
u32 gTypeCurrent = 0;
u32 gFunctionNumber = 0;
u32 gFunctionCurrent = 0;

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

struct HlbcNative {
    HlbcRefString lib;
    HlbcRefString name;
    HlbcRefType type;
    HlbcRefFun findex;
};

struct HlbcType {
    gTypeCurrent += 1;
    std::print("Class: {}/{}", gTypeCurrent, gTypeNumber);
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

struct HlbcOpcode {
    u8 op;
    if (op == 0) { // Mov
        HlbcRefType dst;
        HlbcRefType src;
    } else if (op == 1) { // Int
        HlbcRefType dst;
        HlbcRefType ptr;
    } else if (op == 2) { // Float
        HlbcRefType dst;
        HlbcRefType ptr;
    } else if (op == 3) { // Bool
        HlbcRefType dst;
        HlbcRefType value;
    } else if (op == 4) { // Bytes
        HlbcRefType dst;
        HlbcRefType ptr;
    } else if (op == 5) { // String
        HlbcRefType dst;
        HlbcRefType ptr;
    } else if (op == 6) { // Null
        HlbcRefType dst;
    } else if (op == 7) { // Add
        HlbcRefType dst;
        HlbcRefType a;
        HlbcRefType b;
    } else if (op == 8) { // Sub
        HlbcRefType dst;
        HlbcRefType a;
        HlbcRefType b;
    } else if (op == 9) { // Mul
        HlbcRefType dst;
        HlbcRefType a;
        HlbcRefType b;
    } else if (op == 10) { // SDiv
        HlbcRefType dst;
        HlbcRefType a;
        HlbcRefType b;
    } else if (op == 11) { // UDiv
        HlbcRefType dst;
        HlbcRefType a;
        HlbcRefType b;
    } else if (op == 12) { // SMod
        HlbcRefType dst;
        HlbcRefType a;
        HlbcRefType b;
    } else if (op == 13) { // UMod
        HlbcRefType dst;
        HlbcRefType a;
        HlbcRefType b;
    } else if (op == 14) { // Shl
        HlbcRefType dst;
        HlbcRefType a;
        HlbcRefType b;
    } else if (op == 15) { // SShr
        HlbcRefType dst;
        HlbcRefType a;
        HlbcRefType b;
    } else if (op == 16) { // UShr
        HlbcRefType dst;
        HlbcRefType a;
        HlbcRefType b;
    } else if (op == 17) { // And
        HlbcRefType dst;
        HlbcRefType a;
        HlbcRefType b;
    } else if (op == 18) { // Or
        HlbcRefType dst;
        HlbcRefType a;
        HlbcRefType b;
    } else if (op == 19) { // Xor
        HlbcRefType dst;
        HlbcRefType a;
        HlbcRefType b;
    } else if (op == 20) { // Neg
        HlbcRefType dst;
        HlbcRefType src;
    } else if (op == 21) { // Not
        HlbcRefType dst;
        HlbcRefType src;
    } else if (op == 22) { // Incr
        HlbcRefType dst;
    } else if (op == 23) { // Decr
        HlbcRefType dst;
    } else if (op == 24) { // Call0
        HlbcRefType dst;
        HlbcRefFun fun;
    } else if (op == 25) { // Call1
        HlbcRefType dst;
        HlbcRefFun fun;
        HlbcRefFun arg0;
    } else if (op == 26) { // Call2
        HlbcRefType dst;
        HlbcRefFun fun;
        HlbcRefFun arg0;
        HlbcRefFun arg1;
    } else if (op == 27) { // Call3
        HlbcRefType dst;
        HlbcRefFun fun;
        HlbcRefFun arg0;
        HlbcRefFun arg1;
        HlbcRefFun arg2;
    } else if (op == 28) { // Call4
        HlbcRefType dst;
        HlbcRefFun fun;
        HlbcRefFun arg0;
        HlbcRefFun arg1;
        HlbcRefFun arg2;
        HlbcRefFun arg3;
    } else if (op == 29) { // CallN
        HlbcRefType dst;
        HlbcRefFun fun;
        HlbcVar nargs;
        if (nargs.value > 0) {
            HlbcRefType args[nargs.value];
        }
    } else if (op == 30) { // CallMethod
        HlbcRefType dst;
        HlbcRefField field;
        HlbcVar nargs;
        if (nargs.value > 0) {
            HlbcRefType args[nargs.value];
        }
    } else if (op == 31) { // CallThis
        HlbcRefType dst;
        HlbcRefField field;
        HlbcVar nargs;
        if (nargs.value > 0) {
            HlbcRefType args[nargs.value];
        }
    } else if (op == 32) { // CallClosure
        HlbcRefType dst;
        HlbcRefType fun;
        HlbcVar nargs;
        if (nargs.value > 0) {
            HlbcRefType args[nargs.value];
        }
    } else if (op == 33) { // StaticClosure
        HlbcRefType dst;
        HlbcRefFun fun;
    } else if (op == 34) { // InstanceClosure
        HlbcRefType dst;
        HlbcRefFun fun;
        HlbcRefType obj;
    } else if (op == 35) { // VirtualClosure
        HlbcRefType dst;
        HlbcRefType obj;
        HlbcRefType field;
    } else if (op == 36) { // GetGlobal
        HlbcRefType dst;
        HlbcRefType global;
    } else if (op == 37) { // SetGlobal
        HlbcRefType global;
        HlbcRefType src;
    } else if (op == 38) { // Field
        HlbcRefType dst;
        HlbcRefType obj;
        HlbcRefField field;
    } else if (op == 39) { // SetField
        HlbcRefType obj;
        HlbcRefField field;
        HlbcRefType src;
    } else if (op == 40) { // GetThis
        HlbcRefType dst;
        HlbcRefField field;
    } else if (op == 41) { // SetThis
        HlbcRefField field;
        HlbcRefType src;
    } else if (op == 42) { // DynGet
        HlbcRefType dst;
        HlbcRefType obj;
        HlbcRefString field;
    } else if (op == 43) { // DynSet
        HlbcRefType obj;
        HlbcRefString field;
        HlbcRefType src;
    } else if (op == 44) { // JTrue
        HlbcRefType cond;
        HlbcRefType offset;
    } else if (op == 45) { // JFalse
        HlbcRefType cond;
        HlbcRefType offset;
    } else if (op == 46) { // JNull
        HlbcRefType reg;
        HlbcRefType offset;
    } else if (op == 47) { // JNotNull
        HlbcRefType reg;
        HlbcRefType offset;
    } else if (op == 48) { // JSLt
        HlbcRefType a;
        HlbcRefType b;
        HlbcRefType offset;
    } else if (op == 49) { // JSGte
        HlbcRefType a;
        HlbcRefType b;
        HlbcRefType offset;
    } else if (op == 50) { // JSGt
        HlbcRefType a;
        HlbcRefType b;
        HlbcRefType offset;
    } else if (op == 51) { // JSLte
        HlbcRefType a;
        HlbcRefType b;
        HlbcRefType offset;
    } else if (op == 52) { // JULt
        HlbcRefType a;
        HlbcRefType b;
        HlbcRefType offset;
    } else if (op == 53) { // JUGte
        HlbcRefType a;
        HlbcRefType b;
        HlbcRefType offset;
    } else if (op == 54) { // JNotLt
        HlbcRefType a;
        HlbcRefType b;
        HlbcRefType offset;
    } else if (op == 55) { // JNotGte
        HlbcRefType a;
        HlbcRefType b;
        HlbcRefType offset;
    } else if (op == 56) { // JEq
        HlbcRefType a;
        HlbcRefType b;
        HlbcRefType offset;
    } else if (op == 57) { // JNotEq
        HlbcRefType a;
        HlbcRefType b;
        HlbcRefType offset;
    } else if (op == 58) { // JAlways
        HlbcRefType offset;
    } else if (op == 59) { // ToDyn
        HlbcRefType dst;
        HlbcRefType src;
    } else if (op == 60) { // ToSFloat
        HlbcRefType dst;
        HlbcRefType src;
    } else if (op == 61) { // ToUFloat
        HlbcRefType dst;
        HlbcRefType src;
    } else if (op == 62) { // ToInt
        HlbcRefType dst;
        HlbcRefType src;
    } else if (op == 63) { // SafeCast
        HlbcRefType dst;
        HlbcRefType src;
    } else if (op == 64) { // UnsafeCast
        HlbcRefType dst;
        HlbcRefType src;
    } else if (op == 65) { // ToVirtual
        HlbcRefType dst;
        HlbcRefType src;
    } else if (op == 66) { // Label
    } else if (op == 67) { // Ret
        HlbcRefType ret;
    } else if (op == 68) { // Throw
        HlbcRefType exc;
    } else if (op == 69) { // Rethrow
        HlbcRefType exc;
    } else if (op == 70) { // Switch
        HlbcRefType reg;
        HlbcVar Noffsets;
        if (Noffsets.value > 0) {
            HlbcRefType offsets[Noffsets.value];
        }
        HlbcRefType end;
    } else if (op == 71) { // NullCheck
        HlbcRefType reg;
    } else if (op == 72) { // Trap
        HlbcRefType exc;
        HlbcRefType offset;
    } else if (op == 73) { // EndTrap
        HlbcRefType exc;
    } else if (op == 74) { // GetI8
        HlbcRefType dst;
        HlbcRefType bytes;
        HlbcRefType index;
    } else if (op == 75) { // GetI16
        HlbcRefType dst;
        HlbcRefType bytes;
        HlbcRefType index;
    } else if (op == 76) { // GetMem
        HlbcRefType dst;
        HlbcRefType bytes;
        HlbcRefType index;
    } else if (op == 77) { // GetArray
        HlbcRefType dst;
        HlbcRefType array;
        HlbcRefType index;
    } else if (op == 78) { // SetI8
        HlbcRefType bytes;
        HlbcRefType index;
        HlbcRefType src;
    } else if (op == 79) { // SetI16
        HlbcRefType bytes;
        HlbcRefType index;
        HlbcRefType src;
    } else if (op == 80) { // SetMem
        HlbcRefType bytes;
        HlbcRefType index;
        HlbcRefType src;
    } else if (op == 81) { // SetArray
        HlbcRefType array;
        HlbcRefType index;
        HlbcRefType src;
    } else if (op == 82) { // New
        HlbcRefType dst;
    } else if (op == 83) { // ArraySize
        HlbcRefType dst;
        HlbcRefType array;
    } else if (op == 84) { // Type
        HlbcRefType dst;
        HlbcRefType ty;
    } else if (op == 85) { // GetType
        HlbcRefType dst;
        HlbcRefType src;
    } else if (op == 86) { // GetTID
        HlbcRefType dst;
        HlbcRefType src;
    } else if (op == 87) { // Ref
        HlbcRefType dst;
        HlbcRefType src;
    } else if (op == 88) { // Unref
        HlbcRefType dst;
        HlbcRefType src;
    } else if (op == 89) { // Setref
        HlbcRefType dst;
        HlbcRefType value;
    } else if (op == 90) { // MakeEnum
        HlbcRefType dst;
        HlbcRefType construct;
        HlbcVar nargs;
        if (nargs.value > 0) {
            HlbcRefType args[nargs.value];
        }
    } else if (op == 91) { // EnumAlloc
        HlbcRefType dst;
        HlbcRefType construct;
    } else if (op == 92) { // EnumIndex
        HlbcRefType dst;
        HlbcRefType value;
    } else if (op == 93) { // EnumField
        HlbcRefType dst;
        HlbcRefType value;
        HlbcRefType construct;
        HlbcRefField field;
    } else if (op == 94) { // SetEnumField
        HlbcRefType value;
        HlbcRefField field;
        HlbcRefType src;
    } else if (op == 95) { // Assert
    } else if (op == 96) { // RefData
        HlbcRefType dst;
        HlbcRefType src;
    } else if (op == 97) { // RefOffset
        HlbcRefType dst;
        HlbcRefType src;
        HlbcRefType offset;
    } else if (op == 98) { // Nop
    } else if (op == 99) { // Prefetch
        HlbcRefType value;
        HlbcRefField field;
        HlbcRefType mode;
    } else if (op == 100) { // Asm
        HlbcRefType mode;
        HlbcRefType value;
        HlbcRefType reg;
    } else {
        std::print("Unknown instruction at address {}", addressof(this));
    }
};

struct HlbcFunctionDebug {
    u32 currfile = 0 [[export]];
    u32 cnt = 0 [[export]];
    u32 size = 0 [[export]];
};

fn setup_function_debug(u64 addr, u32 nops, ref HlbcFunctionDebug fdebug) {
    u32 i = 0;
    u8 c;
    u32 currfile;
    u32 cnt;
    u32 offset = 0;
    while (i < nops) {
        c = read_byte(addr);
        addr += 1;
        offset += 1;
        if ((c & 1) != 0)
        {
            c >>= 1;
            currfile = ((c << 8) | read_byte(addr));
            addr += 1;
            offset += 1;
        } else if ((c & 2) != 0) {
            cnt = (c >> 2) & 15;
            while (cnt > 0) {
                cnt -= 1;
                i += 1;
            }
        } else if ((c & 4) != 0) {
            i += 1;
        } else {
            addr += 2;
            offset += 2;
            i += 1;
        }
    }

    fdebug.currfile = currfile;
    fdebug.cnt = cnt;
    fdebug.size = offset;
};

struct HlbcAssign {
    HlbcRefString x;
    HlbcVar y;
};

struct HlbcFunction {
    gFunctionCurrent += 1;
    std::print("Function: {}/{}", gFunctionCurrent, gFunctionNumber);
    HlbcRefType type;
    HlbcRefFun findex;
    HlbcVar nregs;
    HlbcVar nops;
    if (nregs.value > 0) {
        HlbcRefType regs[nregs.value];
    }
    if (nops.value > 0) {
        HlbcOpcode ops[nops.value];
    }
    if (gHasDebug != 0) {
        HlbcFunctionDebug fdebug;
        setup_function_debug($, nops.value, fdebug);
        $ += fdebug.size;
    }
    if ((gHasDebug != 0) && (gVersion >= 3)) {
        HlbcVar nassigns;
        if (nassigns.value > 0) {
            HlbcAssign assigns[nassigns.value];
        }
    }
};

struct HlbcConstantDef {
    HlbcRefGlobal global;
    HlbcVar nfields;
    if (nfields.value > 0) {
        HlbcVar fields[nfields.value];
    }
};

struct StrHlbc {
    char s[];
};

struct Hlbc {
    char magic[3];
    s8 version;
    gVersion = version;
    HlbcVar flags;
    HlbcVar nints;
    HlbcVar nfloats;
    HlbcVar nstrings;
    if (version >= 4) {
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
    if (version >= 4) {
        u32 byte_data;
        u8 bytes[byte_data];
        HlbcVar byte_sizes[nbytes.value];
    }
    if (flags.value & 1 == 1) {
        HlbcVar ndebug_files;
        u32 file_data;
        char files[file_data];
        HlbcVar file_sizes[ndebug_files.value];
        gHasDebug = 1;
    }
    gTypeNumber = ntypes.value;
    if (ntypes.value > 0) {
        HlbcType types[ntypes.value];
    }
    if (nglobals.value > 0) {
        HlbcRefGlobal globals[nglobals.value];
    }
    if (nnatives.value > 0) {
        HlbcNative natives[nnatives.value];
    }
    gFunctionNumber = nfunctions.value;
    if (nfunctions.value > 0) {
        HlbcFunction functions[nfunctions.value];
    }
    if (nconstants.value > 0) {
        HlbcConstantDef constants[nconstants.value];
    }
};

Hlbc hlbc @ 0x00;
