# Test: As long as we don't send an NMI EOI, even after we IRET,
#       we cannot recieve another NMI

MOV $0x8000 %rbp
IOW $0x0001 %rbp

MOV $0 %ra
ST data.int_count %ra

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

        LD data.int_count %ra
        CMP $1 %ra
        JNE fail

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



fail:
    ABRT

    int1:
        LIHP int2

        LD data.int_count %ra
        ADD $1 %ra
        ST data.int_count %ra

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

        IRET

    int2:
        ABRT

data.int_count:
    NOP