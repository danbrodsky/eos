
    # Goals:
    # 1. pick a bootloader CPU
    # 2. clear BSS section to 0
    # 3. drop into an address location where our code lives


    # do not compress instructons
    .option norvc

    # no initalized variables so empty data
    .section .data


    .section .text.init
    # mark as exported symbol so linker knows where _start is and loads here
    .global _start
_start:
    # control status register read - read hardware thread status into temp register
    csrr    t0, mhartid
    # branch not equal zero - if prev val not 0, go to label 3
    bnez    t0, 3f

    # write 0 to CSR satp ( Supervisor Address Translation and Protection register) to disable it
    # since we have no MMU controller yet
    # zero is a hard-wired zero, more efficient than 0
    csrw    satp, zero

    # make a temporary change with .option push
    .option push
    # don't relax code sequences, keep normal instruction length
    .option norelax

    # load address of _global_pointer into gp (global pointer) register
    # global pointer is used to optimize mem access by making all addressing
    # gp-relative, which speeds up execution if gp is near most-used ram
    la      gp, _global_pointer

    # restore normal execution
    .option pop

    # BSS section
    # load _bss_start into register a0
    la      a0, _bss_start
    la      a1, _bss_end
    # branch 2 forward if a0 is greater than or equal to a1 (unsigned)
    bgeu    a0, a1, 2f
1:
    # store DW (double-word (64-bits)) 0 into location at a0 (_bss_start)
    sd      zero, (a0)
    # add together a0 and 8, store result in a0
    addi    a0, a0, 8
    # branch if a0 < a1 (unsigned)
    bltu    a0, a1, 1b
2:

    # li t5, 0xffff
    # csrw medeleg, t5
    # csrw mideleg, t5

    # set stack pointer to stack area defined by linker
    la      sp, _stack

    # load
    li      t0, (0b11 << 11) | (1 << 7) | (1 << 3)
    # set machine status with flags:
    # 12-11 - machine mode enabled
    # 7, 3  - additional interrupts enabled
    csrw    mstatus, t0
    # write kmain (kernel main) address into t1
    la      t1, kmain
    # write kernel main address into machine exception program counter
    csrw    mepc, t1
    la      t2, asm_trap_vector
    # write machine trap vector in
    # this function is called whenever a trap occurs
    # and is the entry point for the interrupt routine
    csrw    mtvec, t2
    li      t3, (1 << 3)|(1 << 7)| (1 << 11)
    # set machine interrupt enable to enable particular interrupts
    csrw    mie, t3
    la      ra, 4f
    # jump back through mepc, which is now set to kernel main function
    mret
3:
4:
    # Wait for interrupt/event
    wfi
    # jump back to label 3 (blocking call)
    j       4b
