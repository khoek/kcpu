# Test: (NMI BIT)
# 1. ENABLE+Triggered is recieved,
# 2. DISABLE+Triggered is recieved.

MOV $0x8000 %rbp
IOW $0x0001 %rbp

step1:
    LIHP int1
    JMP+EI step1.main

    step1.main:
        MOV $0xC001 %ra
        IOW $0x0001 %ra

        NOP
        NOP
        NOP
        NOP
        NOP
        NOP
        NOP
        NOP
        NOP

        CMP $0xFFFF %ra
        JE step2

        ABRT

    int1:
        MOV $0x4000 %rbp
        IOW $0x0001 %rbp

        MOV $0xFFFF %ra
        IRET

step2:
    LIHP int2
    JMP+DI step2.main

    step2.main:
        MOV $0xC001 %ra
        IOW $0x0001 %ra

        NOP
        NOP
        NOP
        NOP
        NOP
        NOP
        NOP
        NOP
        NOP

        ABRT

    int2:
        HLT
