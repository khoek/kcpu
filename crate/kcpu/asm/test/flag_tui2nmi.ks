LIHP int_handle

MOV $0x0 %ra
ST data.nmi_count %ra

# Enable jumper TUI2NMI
IOR $0xD0 %ra
OR $0x0001 %ra
IOW $0xD0 %ra

### Instruction counting starts now ###

    # Do something

    # Check that NOPs do not infinite loop (due to implementation subtleties, because they raise the INSTMASK)
    NOP
    NOP
    NOP
    NOP
    NOP

    # Check that we can do other stuff
    MOV $0xDEAD %ra
    MOV $0xBEEF %rb
    ADD %rb %ra
    XOR %ra %rb
    PUSH %ra
    PUSH %rb
    POP %ra
    POP %rb

    CMP $0x2373 %ra
    JNE fail
    CMP $0x9D9C %rb
    JNE fail

    # Disable jumper TUI2NMI
    IOR $0xD0 %ra
    AND $0xFFFE %ra
    IOW $0xD0 %ra

### Instruction counting ends now ###

# Read how many instructions were counted
LD data.nmi_count %ra
# The correct answer is 20
CMP $20 %ra
JNE fail

HLT

fail:
    ABRT

int_handle:
    PUSHA

    # Check that we are in an NMI
    IOR $0x01 %ra
    CMP $0x0001 %ra
    JNE fail

    # Increment NMI count
    LD data.nmi_count %ra
    INC %ra
    ST data.nmi_count %ra

    # Issue EOI
    MOV $0x4000 %ra
    IOW $0x01 %ra

    POPA
    IRET

data.nmi_count:
    NOP