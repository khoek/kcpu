use crate::spec::types::{hw::{Word, IU}, schema::{Segment, OpClass}};

// START PREAMBLE

// RUSTFIX this is now officially a hack
pub const fn ITFLAG(bits: Segment) -> Word { bits << OpClass::ITYPE_SHIFT }
pub const fn ICFLAG(bits: Segment) -> Word { bits << 0 }
// RUSTFIX make this return an `Iu3Prefix`
pub const fn ICFLAG_IU3(bits: Segment) -> Word { bits << IU::WIDTH }

// END PREAMBLE


// RUSTFIX remove? a newer version of this message which is more clarifyuing is in `types`

/*
    There are 10 opcode bits excliding IU1 and IU2. We allocate
    the highest for LOADDATA, leaving 9. (We are
    happy at the moment to overlap IU3 into the opcode range.);

    Also, for previous design reasons the highest of these 9 bits
    is completely unused, but this is free to change. Below we
    denote LOADDATA by 'C' and this unused bit by '?'.

    We divide the opcode space into 0bC?AAAABBBB, with AAAA the
    "itype" and BBBB the "icode". Typically the low 3 bits
    of icode are required for a fixed itype, and the fourth
    bit specifies a flag. If for a fixed semantic "instruction
    type" we need more then 4 bits for the options (say, for
    a flag), then we just use two icodes.

    Prefix flag (C):
    NOTE This bit is already shifted into its final position in
    the instruction, unlike the bits for AAAA and BBBB below.
*/
// RUSTFIX ALREADY IN hw.rs. Refrence in comment above?
// pub const P_I_LOADDATA (1 << 15);

// itype ranges (AAAA):
pub const IT_CTL    : Segment =           0b0000; // CTL (NOP, INT must occupy hardcoded positions)
pub const IT_STK    : Segment =           0b0001; // STK (Stack manipulation, call/return, IRET, etc.)
pub const IT__MEMC  : Segment =           0b0010; // MEM (CLOSE, don't use directly, use IT_MEM and ITFLAG_MEM_FAR instead)
pub const IT__MEMF  : Segment =           0b0011; // MEM (FAR, don't use directly, use IT_MEM and ITFLAG_MEM_FAR instead)
pub const IT__JMP   : Segment =           0b0100; // JMP (don't use directly, use IT_JMP and ITFLAG_JMP_LD instead)
pub const IT__JMPLD : Segment =           0b0101; // JMP (don't use directly, use IT_JMP and ITFLAG_JMP_LD instead)
pub const IT_ALU1   : Segment =           0b0110; // ALU (insts with a NF (noflags) variant)
pub const IT_ALU2   : Segment =           0b0111; // ALU (other ALU insts)
// reserved itypes for IU3_ALL/_SINGLE opclasses
pub const IT_IU3_ALL_GRP1: Segment =      0b1000;
pub const IT_IU3_ALL_GRP2: Segment =      0b1001;
pub const IT_IU3_ALL_GRP3: Segment =      0b1010;
// Don't forget that the `GCTRL_NRM_IU3_OVERRIDE_O_SELECT_RSP_I__UNUSED` mechanism exists largely
// to prevent the creation of IU3_SINGLE opclassess.
// pub const IT_IU3_SINGLE_GRP1 0b1000 // It is very wasteful to have a separate itype for IU3_SINGLEs, but lets just be lazy for now.

// RUSTFIX implement these type safely. Could have a struct which implements trait IType, 
// which stores the specific flags which that IType can have, and reserves them at initial
// registration time to prevent itype collisions.

// Fake ICs (to implement flags) and flags at the itype/icode level
pub const IT_MEM : Segment = 0b0010;
pub const IT_JMP : Segment = 0b0100;
pub const ITFLAG_MEM_FAR     : Segment = ITFLAG(0b0001);
pub const ITFLAG_JMP_LD      : Segment = ITFLAG(0b0001);
pub const ICFLAG_ALU1_NOFGS  : Segment = ICFLAG(0b1000);
pub const ICFLAG_MEM_IU3_FAR : Segment = ICFLAG_IU3(0b1);
pub const ICFLAG_ADD3_IU3_NF : Segment = ICFLAG_IU3(0b1);

// BEGIN DECLS

// CTL/MISC (12/16)
pub const I_NOP       : OpClass = OpClass::new(IT_CTL, 0b0000);
pub const I__DO_INT   : OpClass = OpClass::new(IT_CTL, 0b0001);

pub const I_MOV       : OpClass = OpClass::new(IT_CTL, 0b0011);
pub const I_LCFG      : OpClass = OpClass::new(IT_CTL, 0b0100);
pub const I_LFG       : OpClass = OpClass::new(IT_CTL, 0b0101);
pub const I_LIHP      : OpClass = OpClass::new(IT_CTL, 0b0110);

pub const I_IOR       : OpClass = OpClass::new(IT_CTL, 0b1000);
pub const I_IOW       : OpClass = OpClass::new(IT_CTL, 0b1001);

pub const I_DI        : OpClass = OpClass::new(IT_CTL, 0b1100);
pub const I_EI        : OpClass = OpClass::new(IT_CTL, 0b1101);

pub const I_HLT       : OpClass = OpClass::new(IT_CTL, 0b1110);
pub const I_ABRT      : OpClass = OpClass::new(IT_CTL, 0b1111);

// STK (12/16)
pub const I_PUSH      : OpClass = OpClass::new(IT_STK, 0b0000);
pub const I_POP       : OpClass = OpClass::new(IT_STK, 0b0001);
pub const I_PUSHx2    : OpClass = OpClass::new(IT_STK, 0b0010);
pub const I_POPx2     : OpClass = OpClass::new(IT_STK, 0b0011);
pub const I_PUSHFG    : OpClass = OpClass::new(IT_STK, 0b0100);
pub const I_POPFG     : OpClass = OpClass::new(IT_STK, 0b0101);
pub const I_CALL      : OpClass = OpClass::new(IT_STK, 0b0110);
pub const I_RET       : OpClass = OpClass::new(IT_STK, 0b0111);

pub const I_IRET      : OpClass = OpClass::new(IT_STK, 0b1000);
pub const I_ENTER1    : OpClass = OpClass::new(IT_STK, 0b1001);
pub const I_ENTERFR2  : OpClass = OpClass::new(IT_STK, 0b1010);
pub const I_LEAVE1    : OpClass = OpClass::new(IT_STK, 0b1011);

// ALU1 (NF-variant possible) (8/8)
pub const I_ADD2      : OpClass = OpClass::new(IT_ALU1, 0b0000);
pub const I_SUB       : OpClass = OpClass::new(IT_ALU1, 0b0001);
pub const I_BSUB      : OpClass = OpClass::new(IT_ALU1, 0b0010);
pub const I_AND       : OpClass = OpClass::new(IT_ALU1, 0b0011);
pub const I_OR        : OpClass = OpClass::new(IT_ALU1, 0b0100);
pub const I_XOR       : OpClass = OpClass::new(IT_ALU1, 0b0101);
pub const I_LSFT      : OpClass = OpClass::new(IT_ALU1, 0b0110);
pub const I_RSFT      : OpClass = OpClass::new(IT_ALU1, 0b0111);

// ALU2 (2/16)
pub const I_TST       : OpClass = OpClass::new(IT_ALU2, 0b0000);
pub const I_CMP       : OpClass = OpClass::new(IT_ALU2, 0b0001);

// MEM (9/16)
pub const I_STPFX     : OpClass = OpClass::new(IT_MEM, 0b0001);
pub const I_LDW       : OpClass = OpClass::new(IT_MEM, 0b0011);
pub const I_LDBL      : OpClass = OpClass::new(IT_MEM, 0b0100);
pub const I_LDBH      : OpClass = OpClass::new(IT_MEM, 0b0110);
pub const I_LDBLZ     : OpClass = OpClass::new(IT_MEM, 0b0101);
pub const I_LDBHZ     : OpClass = OpClass::new(IT_MEM, 0b0111);
pub const I_STW       : OpClass = OpClass::new(IT_MEM, 0b1011);
pub const I_STBL      : OpClass = OpClass::new(IT_MEM, 0b1100);
pub const I_STBH      : OpClass = OpClass::new(IT_MEM, 0b1110);

// JMP (12/16)
pub const I_JC        : OpClass = OpClass::new(IT_JMP, 0b0000);
pub const I_JNC       : OpClass = OpClass::new(IT_JMP, 0b0100);
pub const I_JZ        : OpClass = OpClass::new(IT_JMP, 0b0001);
pub const I_JNZ       : OpClass = OpClass::new(IT_JMP, 0b0101);
pub const I_JS        : OpClass = OpClass::new(IT_JMP, 0b0010);
pub const I_JNS       : OpClass = OpClass::new(IT_JMP, 0b0110);
pub const I_JO        : OpClass = OpClass::new(IT_JMP, 0b0011);
pub const I_JNO       : OpClass = OpClass::new(IT_JMP, 0b0111);

pub const I_JMP       : OpClass = OpClass::new(IT_JMP, 0b1000);
pub const I_LJMP      : OpClass = OpClass::new(IT_JMP, 0b1001);

pub const I_JMP_DI    : OpClass = OpClass::new(IT_JMP, 0b1110);
pub const I_JMP_EI    : OpClass = OpClass::new(IT_JMP, 0b1111);

// IU3_ALL_GRP1 (2/2);
pub const I_ADD3      : OpClass = OpClass::with_iu3_all(IT_IU3_ALL_GRP1, 0b0);
pub const I_ADD3NF    : OpClass = OpClass::with_iu3_all(IT_IU3_ALL_GRP1, 0b1); // REMINDER UNREFERENCED, use I_ADD3 and ICFLAG_ADD3_IU3_NF instead.

// IU3_ALL_GRP2 (2/2);
pub const I_LDWO      : OpClass = OpClass::with_iu3_all(IT_IU3_ALL_GRP2, 0b0);
pub const I_LDWO_FAR  : OpClass = OpClass::with_iu3_all(IT_IU3_ALL_GRP2, 0b1); // REMINDER UNREFERENCED, use I_LDWO and ICFLAG_MEM_IU3_FAR instead.

// IU3_ALL_GRP3 (2/2);
pub const I_STWO      : OpClass = OpClass::with_iu3_all(IT_IU3_ALL_GRP3, 0b0);
pub const I_STWO_FAR  : OpClass = OpClass::with_iu3_all(IT_IU3_ALL_GRP3, 0b1); // REMINDER UNREFERENCED, use I_STWO and ICFLAG_MEM_IU3_FAR instead.

// END DECLS
