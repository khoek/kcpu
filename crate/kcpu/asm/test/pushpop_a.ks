

MOV $0x1337 %ra
MOV $0xDEAD %rb
MOV $0xF00F %rc
MOV $0x0FF0 %rd
MOV $0xD10D %re
MOV $0x7154 %rbp
# RID has to come last
MOV $0xBEEF %rid

PUSHA

MOV $0xFFFF %ra
MOV $0xFFFF %rb
MOV $0xFFFF %rc
MOV $0xFFFF %rd
MOV $0xFFFF %re
MOV $0xFFFF %rbp
MOV $0xFFFF %rid

POPA

# We don't use CMP directly here since RID will be overridden when the constant is loaded.
PUSH %ra
MOV %rid %ra
CMP $0xBEEF %ra
# IMPORTANT NOTE: Preservation of RID has been disabled, since no aliases use RID between
# instructions. Re-enable this if we add aliases which do (since otherwise we would have
# register state corruption if an interrupt came in at an inopportune time).
#JNZ fail
POP %ra

CMP $0x7154 %rbp
JNZ fail

CMP $0xD10D %re
JNZ fail

CMP $0x0FF0 %rd
JNZ fail

CMP $0xF00F %rc
JNZ fail

CMP $0xDEAD %rb
JNZ fail

CMP $0x1337 %ra
JNZ fail


HLT


fail:
    ABRT