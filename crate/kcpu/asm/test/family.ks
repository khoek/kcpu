XOR %ra %ra
MOV $0xBEEF %rb
ST %ra $0xBEEF

LD %rsp %ra
CMP $0xBEEF %ra
JNE fail

MOV $0x1337 %ra
LD %rsp %la
CMP $0x13EF %ra
JNE fail

MOV $0x1337 %ra
LD %rsp %ha
CMP $0xEF37 %ra
JNE fail

MOV $0x1337 %ra
LDZ %rsp %la
CMP $0x00EF %ra
JNE fail

MOV $0x1337 %ra
LDZ %rsp %ha
CMP $0xEF00 %ra
JNE fail

HLT

fail:
    ABRT