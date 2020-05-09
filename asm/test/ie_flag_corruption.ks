


LIHP int_handle

# Enable jumper TUI2NMI
IOR $0xD0 %ra
OR $0x0001 %ra
IOW $0xD0 %ra

### NMIs start now ###

MOV $1 %ra
MOV $1 %rb

sub_loop:
    CMP %ra %rb
    JL done
    SUB %rb %ra
    JMP sub_loop

done:
    HLT


int_handle:
    PUSH %ra

    # Issue EOI
    MOV $0x4000 %ra
    IOW $0x01 %ra

    POP %ra
    IRET

