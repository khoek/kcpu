
MOV $54 %rbp
MOV $0 %rb

loop:
    INC %rsp

    IOW $0xC3 %rb

    MOV $0x33 %rd
    AND %rb %rd
    JNZ skip_mod

    ADD $222 %rbp

skip_mod:
    IOR $0xC4 %rbp
    ADD %rb %rbp
    IOW $0xC4 %rbp
    MOV $0x01 %ra
    IOW $0xC0 %ra

    IOR $0xC4 %rbp
    ADD %rb %rbp
    IOW $0xC4 %rbp
    MOV $0x01 %ra
    IOW $0xC0 %ra

    ADD $1 %rb

    CMP $0x2000 %rsp
    JE done

    JMP loop

done:
    HLT