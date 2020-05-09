LIHP int

MOV $0x8006 %rbp
IOW $0x0001 %rbp

JMP+EI main

main:
    MOV $0xC004 %ra
    IOW $0x0001 %ra

    ABRT

int:
    HLT