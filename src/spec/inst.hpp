#ifndef SPEC_INST_H
#define SPEC_INST_H

#include "opclass.hpp"

namespace kcpu {

// START PREAMBLE

#define INST_SHIFT (2 * IU_WIDTH)
#define INST_STRIP_IU3(raw) ((raw) & ~IU_MASK)
#define INST_GET_LOADDATA(inst) ((inst) & P_I_LOADDATA)
#define INST_GET_OPCODE(inst) (((inst) & ~P_I_LOADDATA) >> INST_SHIFT)
#define INST_MK(loaddata, opcode, iu1, iu2, iu3) (((loaddata) ? P_I_LOADDATA : 0) | ((opcode) << INST_SHIFT) | INST_MK_IU1(iu1) | INST_MK_IU2(iu2) | INST_MK_IU3(iu3))

#define ITYPE_SHIFT 4

#define ITFLAG(bits) ((bits) << ITYPE_SHIFT)
#define ICFLAG(bits) ((bits) << 0)

#define OC(ic, raw) opclass(((ic) << ITYPE_SHIFT) | (raw))
#define OCSINGLE_IU3(ic, raw, iu3) opclass_iu3_single(((ic) << ITYPE_SHIFT) | ((raw) << IU_WIDTH) | (iu3), (iu3))
#define OCANY_IU3(ic, raw) opclass_iu3_all(((ic) << ITYPE_SHIFT) | ((raw) << IU_WIDTH))

// END PREAMBLE

// There are 10 opcode bits excliding IU1 and IU2. We allocate
// the highest two for LOADDATA and RSPDEC, leaving 8. (We are
// happy at the moment to overlap IU3 into the opcode range.)
//
// We divide the opcode space into 0bCCAAAABBBB, with AAAA the
// "itype" and BBBB the "icode". Typically the low 3 bits
// of icode are required for a fixed itype, and the fourth
// bit specifies a flag. If for a fixed semantic "instruction
// type" we need more then 4 bits for the options (say, for
// a flag), then we just use two icodes.
//
// prefix flags (CC):
// NOTE There first two codes are already shifted into their
// final position in the instruction.
#define P_I_LOADDATA (1 << 15)
#define P_I_RSPDEC   (1 << 14)

#define P_PRE_I_RSPDEC (1 << 8)

//
// itype ranges (AAAA):
#define IT_SYS              0b0000 // SYS (NOP, INT must occupy hardcoded positions)
#define IT_X                0b0001 // X   (X_xxxx codes)
#define IT__MEMC            0b0010 // MEM (CLOSE, don't use directly, use IT_MEM and ITFLAG_MEM_FAR instead)
#define IT__MEMF            0b0011 // MEM (FAR, don't use directly, use IT_MEM and ITFLAG_MEM_FAR instead)
#define IT__JMP             0b0100 // JMP (don't use directly, use IT_JMP and ITFLAG_JMP_LD instead)
#define IT__JMPLD           0b0101 // JMP (don't use directly, use IT_JMP and ITFLAG_JMP_LD instead)
#define IT_ALU              0b0110 // ALU
// reserved itypes for IU3_ALL/_SINGLE opclasses
#define IT_IU3_SINGLE_GRP1  0b0111 // It is very wasteful to have a separate itype for IU3_SINGLEs, but lets just be lazy for now.
#define IT_IU3_ALL_GRP1     0b1000
#define IT_IU3_ALL_GRP2     0b1001
#define IT_IU3_ALL_GRP3     0b1010

// Fake ICs (to implement flags) and flags at the itype/icode level
#define IT_MEM 0b0010
#define IT_JMP 0b0100
#define ITFLAG_MEM_FAR     ITFLAG(0b0001)
#define ITFLAG_JMP_LD      ITFLAG(0b0001)
#define ICFLAG_ALU_NOFGS   ICFLAG(0b1000)
#define ICFLAG_MEM_IU3_FAR ICFLAG(0b1000)

// BEGIN DECLS

// SYS/MISC (12/16)
#define I_NOP       OC(IT_SYS, 0b0000)
#define I__DO_INT   OC(IT_SYS, 0b0001).add_flag(P_PRE_I_RSPDEC)
// #define I__UNUSED   OC(IT_SYS, 0b0010)

#define I_MOV       OC(IT_SYS, 0b0011)
#define I_LCFG      OC(IT_SYS, 0b0100)
#define I_LFG       OC(IT_SYS, 0b0101)
#define I_LIHP      OC(IT_SYS, 0b0110)
// #define I__UNUSED   OC(IT_SYS, 0b0111)

#define I_IOR       OC(IT_SYS, 0b1000)
#define I_IOW       OC(IT_SYS, 0b1001)

#define I_ECRIT     OC(IT_SYS, 0b1100)
#define I_LCRIT     OC(IT_SYS, 0b1101)

#define I_HLT       OC(IT_SYS, 0b1110)
#define I_ABRT      OC(IT_SYS, 0b1111)

// X (8/16)
#define I_X_ENTER   OC(IT_X  , 0b0000).add_flag(P_PRE_I_RSPDEC) // FIXME hack
#define I_X_LEAVE   OC(IT_X  , 0b0001).add_flag(P_PRE_I_RSPDEC) // FIXME hack

#define I_X_PUSH    OC(IT_X  , 0b0010).add_flag(P_PRE_I_RSPDEC) // FIXME hack
#define I_X_POP     OC(IT_X  , 0b0011)
#define I_X_CALL    OC(IT_X  , 0b0100).add_flag(P_PRE_I_RSPDEC) // FIXME hack
#define I_X_RET     OC(IT_X  , 0b0101)

#define I_X_RET_LCRIT OC(IT_X, 0b0110)

#define I_X_PUSHFG  OC(IT_X  , 0b1000).add_flag(P_PRE_I_RSPDEC) // FIXME hack
#define I_X_POPFG   OC(IT_X  , 0b1001)

// ALU (8/8)
#define I_ADD2      OC(IT_ALU, 0b0000)
#define I_SUB       OC(IT_ALU, 0b0001)
#define I_AND       OC(IT_ALU, 0b0010)
#define I_OR        OC(IT_ALU, 0b0011)
#define I_XOR       OC(IT_ALU, 0b0100)
#define I_LSFT      OC(IT_ALU, 0b0101)
#define I_RSFT      OC(IT_ALU, 0b0110)
#define I_TST       OC(IT_ALU, 0b0111)

// MEM (9/16)
#define I_STPFX     OC(IT_MEM, 0b0001)
#define I_LDW       OC(IT_MEM, 0b0011)
#define I_LDBL      OC(IT_MEM, 0b0100)
#define I_LDBH      OC(IT_MEM, 0b0110)
#define I_LDBLZ     OC(IT_MEM, 0b0101)
#define I_LDBHZ     OC(IT_MEM, 0b0111)
#define I_STW       OC(IT_MEM, 0b1011)
#define I_STBL      OC(IT_MEM, 0b1100)
#define I_STBH      OC(IT_MEM, 0b1110)

// JMP (12/16)
#define I_JC        OC(IT_JMP, 0b0000)
#define I_JNC       OC(IT_JMP, 0b0100)
#define I_JZ        OC(IT_JMP, 0b0001)
#define I_JNZ       OC(IT_JMP, 0b0101)
#define I_JS        OC(IT_JMP, 0b0010)
#define I_JNS       OC(IT_JMP, 0b0110)
#define I_JO        OC(IT_JMP, 0b0011)
#define I_JNO       OC(IT_JMP, 0b0111)

#define I_JMP       OC(IT_JMP, 0b1000)
#define I_LJMP      OC(IT_JMP, 0b1001)

#define I_JMP_ECRIT OC(IT_JMP, 0b1110)
#define I_JMP_LCRIT OC(IT_JMP, 0b1111)

// IU3_ALL_GRP1 (1/2)
#define I_ADD3      OCANY_IU3(IT_IU3_ALL_GRP1, 0b0)

// IU3_ALL_GRP2 (2/2)
#define I_LDWO      OCANY_IU3(IT_IU3_ALL_GRP2, 0b0)
#define I_LDWO_FAR  OCANY_IU3(IT_IU3_ALL_GRP2, 0b1) // REMINDER UNREFERENCED, use I_LDWO and ICFLAG_MEM_IU3_FAR instead.

// IU3_ALL_GRP3 (2/2)
#define I_STWO      OCANY_IU3(IT_IU3_ALL_GRP3, 0b0)
#define I_STWO_FAR  OCANY_IU3(IT_IU3_ALL_GRP3, 0b1) // REMINDER UNREFERENCED, use I_LDWO and ICFLAG_MEM_IU3_FAR instead.

// IU3_SINGLE_GRP1 (~1/16)
#define I_X_ENTERFR OCSINGLE_IU3(IT_IU3_SINGLE_GRP1, 0b0, REG_SP).add_flag(P_PRE_I_RSPDEC) // FIXME hack

// END DECLS

}

#endif
