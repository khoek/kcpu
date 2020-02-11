#ifndef SPEC_INST_H
#define SPEC_INST_H

#include "opclass.h"

namespace kcpu {

// START PREAMBLE

#define INST_SHIFT (2 * IU_WIDTH)
#define INST_STRIP_IU3(raw) ((raw) & ~IU_MASK)
#define INST_GET_LOADDATA(inst) ((inst) & P_I_LOADDATA)
#define INST_GET_OPCODE(inst) (((inst) & ~P_I_LOADDATA) >> INST_SHIFT)
#define INST_MK(loaddata, opcode, iu1, iu2, iu3) (((loaddata) ? P_I_LOADDATA : 0) | (opcode << INST_SHIFT) | INST_MK_IU1(iu1) | INST_MK_IU2(iu2) | INST_MK_IU3(iu3))

#define SINGLE_IU3(raw, iu3) opclass_iu3_single((raw) | (iu3), (iu3))
#define ANY_IU3(raw) opclass_iu3_all((raw))

// END PREAMBLE

#define P_I_LOADDATA (1 << 15)

// SYS
#define I_NOP   0b00000000
#define I_HLT   0b01111110
#define I_ABRT  0b01111111

// MEM
#define I_STPFX 0b00000001
#define I_LDW   0b00000011
#define I_LDBL  0b00000100
#define I_LDBH  0b00000110
#define I_LDBLZ 0b00000101
#define I_LDBHZ 0b00000111
#define I_STW   0b00001011
#define I_STBL  0b00001100
#define I_STBH  0b00001110
#define I_STBLZ 0b00001101
#define I_STBHZ 0b00001111
// Use Farmem Prefix
#define P_I_FAR 0b00100000

// CTL
#define I_JMP     0b101101000
#define I_JC      0b101100000
#define I_JNC     0b101100100
#define I_JZ      0b101100001
#define I_JNZ     0b101100101
#define I_JS      0b101100010
#define I_JNS     0b101100110
#define I_JO      0b101100011
#define I_JNO     0b101100111
// LD JMP Prefix
#define P_I_LDJMP 0b00010000

#define I_LJMP    0b00101001

// REG
#define I_MOV   0b00101111

// ALU
#define I_ADD   0b01000000
#define I_SUB   0b01000001
#define I_AND   0b01000010
#define I_OR    0b01000011
#define I_XOR   0b01000100
#define I_LSFT  0b01000101
#define I_RSFT  0b01000110
#define I_TST   0b01000111
// No Flags Prefix
#define P_I_NOFGS 0b00100000

// X
// FIXME? make all of these use IU3 = REG_RSP? not fully sold on IU3 yet.
// TBH its probably best to shrink the microcode count space and then just reserve enough bits for an IU3 as well
#define I_X_PUSH                0b10101000
#define I_X_POP                 0b10101001
#define I_X_CALL                0b10101010
#define I_X_RET                 0b10101011

#define I_X_ENTER               0b10101100
#define I_X_LEAVE               0b10101101
#define I_X_ENTERFR  SINGLE_IU3(0b10101000, REG_SP)

#define I_ADD3          ANY_IU3(0b11000000)
#define I_LDWO          ANY_IU3(0b11001000)
#define I_STWO          ANY_IU3(0b11011000)

}

#endif
