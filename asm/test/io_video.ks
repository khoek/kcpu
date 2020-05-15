
MOV $54 %rbp
MOV $0 %rb

loop:
    INC %rsp

    IOW $0xC2 %rb

    MOV $0x33 %rd
    AND %rb %rd
    JNZ skip_mod

    ADD $222 %rbp

skip_mod:
    IOR $0xC3 %rbp
    ADD %rb %rbp
    IOW $0xC3 %rbp

    ADD $1 %rb

    CMP $0x200 %rsp
    JE done

    JMP loop

done:
    HLT