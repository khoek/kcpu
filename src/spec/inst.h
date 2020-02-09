#ifndef SPEC_INST_H
#define SPEC_INST_H

#define P_I_LOADDATA (1 << 15)

// SYS
#define I_NOP   0b00000000
#define I_HLT   0b01111111

// MEM
#define I_STPFX 0b00000001
#define I_STRIP 0b00000010
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
// Far Prefix bit
#define P_I_FAR 0b00010000

// CTL
#define I_JMP   0b00101000
#define I_JC    0b00100000
#define I_JNC   0b00100100
#define I_JZ    0b00100001
#define I_JNZ   0b00100101
#define I_JS    0b00100010
#define I_JNS   0b00100110
#define I_JO    0b00100011
#define I_JNO   0b00100111
// LD JMP bit
#define P_I_LDJMP 0b00010000

#define I_LJMP  0b00101001

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

#endif
