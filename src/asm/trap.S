# trap.S
# Assembly-level trap handler
    .section .text
    .global asm_trap_vector
asm_trap_vector:
    # used in boot.S as the entry for the interrupt routine
    # currently mepc is set to kernel main so that's where we'll end up
    mret
