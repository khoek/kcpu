# (Same as test number 1, except uses EI/DI instead
#  of combination instructions JMP+EI/JMP+DI.)

# Test:
# 1. ENABLE+Triggered is recieved,
# 2. DISABLE+Triggered is not recieved,
# 3. ENABLE+NOT Triggered is recieved (from previous),
# 4. ENABLE+Triggered is recieved.

MOV $0x8006 %rbp
IOW $0x0001 %rbp

step1:
    LIHP int1
    EI
    JMP step1.main

    step1.main:
        MOV $0xC004 %ra
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
        POP %rbp
        MOV $0x4000 %rbp
        IOW $0x0001 %rbp

        JMP step2

step2:
    LIHP int2
    DI
    JMP step2.main

    step2.main:
        MOV $0xC004 %ra
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

        JMP step3

    int2:
        ABRT

step3:
    LIHP int3
    EI
    JMP step3.main

    step3.main:
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

    int3:
        POP %rbp
        MOV $0x4000 %rbp
        IOW $0x0001 %rbp

        JMP step4

step4:
    LIHP int4
    EI
    JMP step4.main

    step4.main:
        MOV $0xC004 %ra
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

    int4:
        POP %rbp
        MOV $0x4000 %rbp
        IOW $0x0001 %rbp

        HLT