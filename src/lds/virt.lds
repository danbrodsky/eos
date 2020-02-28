OUTPUT_ARCH( "riscv" )

/* start address for executable is at _start */
ENTRY( _start )

MEMORY
{
    /* all memory in ram set to write and execute but no read and */
	/* not initialized */
	ram   (wxa!ri) : ORIGIN = 0x80000000, LENGTH = 128M
}


/* Program Headers */
PHDRS
{
	text PT_LOAD; /* load from file into memory */
	data PT_LOAD;
	bss PT_LOAD;
}

SECTIONS
{
	.text : {
	PROVIDE(_text_start = .); /* write text start to curr addr */
	*(.text.init) *(.text .text.*) /* order by .init first then .text */
	PROVIDE(_text_end = .);
	} >ram AT>ram :text /* >ram - put VMA (virtual mem addr) in ram region */
	/* AT>ram sets LMA (load mem addr) to ram region */
	/* VMA is addr section will have at runtime, LMA is addr section is loaded to */
	/* :text - put this section under .text program header */
	PROVIDE(_global_pointer = .);
	.rodata : {
	PROVIDE(_rodata_start = .);
	*(.rodata .rodata.*)
	PROVIDE(_rodata_end = .);
	} >ram AT>ram :text

	.data : {
	. = ALIGN(4096); /* align page at curr addr to 4096 bytes */
	PROVIDE(_data_start = .);
	*(.sdata .sdata.*) *(.data .data.*)
	PROVIDE(_data_end = .);
	} >ram AT>ram :data

	.bss :{
	PROVIDE(_bss_start = .);
	*(.sbss .sbss.*) *(.bss .bss.*)
	PROVIDE(_bss_end = .);
	} >ram AT>ram :bss

	PROVIDE(_memory_start = ORIGIN(ram));
	PROVIDE(_stack_end = _bss_end + 0x80000);
	PROVIDE(_stack_start = _bss_end);
	PROVIDE(_memory_end = ORIGIN(ram) + LENGTH(ram));
	PROVIDE(_heap_start = _stack_end);
	PROVIDE(_heap_size = _memory_end - _heap_start);
}
