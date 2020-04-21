# NOTE: there are no "zero" variant instructions, like there are for the LDBx series

# at addr 0

MOV $0x1337 %ra
STW $0 %ra
MOV $0xBEEF %ra
STBL $0 %ra
LDW $0 %ra
CMP $0x13EF %ra
JNE fail

MOV $0x1337 %ra
STW $0 %ra
MOV $0xBEEF %ra
STBH $0 %ra
LDW $0 %ra
CMP $0x13BE %ra
JNE fail

# at addr 1

MOV $0x1337 %ra
STW $1 %ra
MOV $0xBEEF %ra
STBL $1 %ra
LDW $0 %ra
CMP $0xEF37 %ra
JNE fail

MOV $0x1337 %ra
STW $1 %ra
MOV $0xBEEF %ra
STBH $1 %ra
LDW $0 %ra
CMP $0xBE37 %ra
JNE fail


HLT



fail:
    ABRT