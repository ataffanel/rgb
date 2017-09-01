    .module	Runtime
    .area	_HEADER (ABS)

    .macro ldhi dest, value
    LD  A, value
    LDH dest, A
    .endm

    .org 0x00
    LD  SP, #0xFFFE
; Initialize registers ...
    XOR A,A
    LDH (0x05), A
    LDH (0x06), A
    LDH (0x07), A

    LD  A, #0x80
    LD  (0x10), A
    LDHI  (0x11), #0xBF
    LDHI  (0x12), #0xF3
    LDHI  (0x14), #0xBF
    LDHI  (0x16), #0x3F
    LDHI  (0x17), #0x00
    LDHI  (0x19), #0xBF
    LDHI  (0x1A), #0x7F
    LDHI  (0x1B), #0xFF
    LDHI  (0x1C), #0x9F
    LDHI  (0x1E), #0xBF
    LDHI  (0x20), #0xFF
    LDHI  (0x21), #0x00
    LDHI  (0x22), #0x00
    LDHI  (0x23), #0xBF
    LDHI  (0x24), #0x77
    LDHI  (0x25), #0xF3
    LDHI  (0x26), #0xF1
    LDHI  (0x40), #0x91
    LDHI  (0x42), #0x00
    LDHI  (0x43), #0x00
    LDHI  (0x45), #0x00
    LDHI  (0x47), #0xFC
    LDHI  (0x48), #0xFF
    LDHI  (0x49), #0xFF
    LDHI  (0x4A), #0x00
    LDHI  (0x4B), #0x00
    LDHI  (0xFF), #0x00

    ; Load register values
    ; Push and pull AF, no math/test operation authorized after this point
    LD  BC, #0x01BC
    PUSH  BC
    POP   AF

    LD  BC, #0x0013
    LD  DE, #0x00D8
    LD  HL, #0x014D


    JP boot

    .org 0xFC
boot:
    LD  A, #1
    LDH (0x50), A
