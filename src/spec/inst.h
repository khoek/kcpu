#ifndef SPEC_INST_H
#define SPEC_INST_H

// START PREAMBLE

#define INST_SHIFT (2 * IU_WIDTH)
#define INST_GET_LOADDATA(inst) ((inst) & P_I_LOADDATA)
#define INST_GET_OPCODE(inst) (((inst) & ~P_I_LOADDATA) >> INST_SHIFT)
#define INST_MK(loaddata, opcode, iu1, iu2, iu3) (((loaddata) ? P_I_LOADDATA : 0) | (opcode << INST_SHIFT) | INST_MK_IU1(iu1) | INST_MK_IU2(iu2) | INST_MK_IU3(iu3))

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
#define P_I_FAR 0b00010000

// CTL
#define I_JMP     0b00101000
#define I_JC      0b00100000
#define I_JNC     0b00100100
#define I_JZ      0b00100001
#define I_JNZ     0b00100101
#define I_JS      0b00100010
#define I_JNS     0b00100110
#define I_JO      0b00100011
#define I_JNO     0b00100111
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
#define I_X_PUSH 0b10101000
#define I_X_POP  0b10101001
#define I_X_CALL 0b10101010
#define I_X_RET  0b10101011

#endif
