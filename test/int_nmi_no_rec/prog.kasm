# Test: Despite being NMIs, we cannot recieve an NMI during an NMI handler
#       (i.e. CBIT_HNMI is working, and prevents an NMI during an NMI handler---
#        we define the end of the NMI handler by the first IRET after it occurs).

MOV $0x8000 %rbp
IOW $0x0001 %rbp

step1:
    LIHP int1
    JMP+DI step1.main

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

        ABRT

    int1:
        LIHP int2

        MOV $0xC001 %ra
        IOW $0x0001 %ra

        MOV $0x4000 %rbp
        IOW $0x0001 %rbp

        NOP
        NOP
        NOP
        NOP
        NOP
        NOP
        NOP
        NOP
        NOP

        HLT

    int2:
        ABRT
