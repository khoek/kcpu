LIHP int

MOV $0x8006 %rbp
IOW $0x0001 %rbp

MOV $0xC004 %ra
IOW $0x0001 %ra

EI

loop:
    JMP loop

int:
    HLT