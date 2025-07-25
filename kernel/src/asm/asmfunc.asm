; asmfunc.asm
; nasm -f elf64 src/asm/asmfunc.asm -o src/asm/asmfunc.o
; System V AMD64 Calling Convention
; Registers: RDI, RSI, RDX, RCX, R8, R9


bits 64
section .text
global IoOut32 ;  void InOut32(uint16_t addr, uint32_t data);
IoOut32:
    mov dx, di   ; dx = addr
    mov eax, esi ; eax = data
    out dx, eax  ; 
    ret

global IoIn32   ; uint32_t IoIn32(uint16_t addr);
IoIn32:
    mov dx, di  ; dx =addr 
    in eax, dx  ; set to eax register from dx register addr
    ret