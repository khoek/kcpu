PUSH $0x01
CALL probe_port
SUB $2 %rsp

TST %ra
JZ fail

PUSH $0x02
CALL probe_port
SUB $2 %rsp

TST %ra
JNZ fail

PUSH $0xF0
CALL probe_port
SUB $2 %rsp

TST %ra
JZ fail

HLT

probe_port:
    LDWO %rsp $2 %rb

    IOW $0x00 %rb
    IOR $0x00 %ra

    RET

fail:
    ABRT