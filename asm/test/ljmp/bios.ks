MOV $0 %ra
ADD $1 %ra
MOV $8 %rb
LJMP $0x80 %rb
MOV $0xFFFF %rb
ADD $1 %ra

CMP $3 %ra
JE win
ABRT

win:
HLT
