OUTPUT_FORMAT("elf64-x86-64","elf64-x86-64","elf64-x86-64");
OUTPUT_ARCH("i386:x86-64")

ENTRY(entry)

PHDRS
{
   phstack PT_LOAD FLAGS (6);
   phsetup PT_LOAD FLAGS (7);
}

SECTIONS
{
   /* 32MB to have enough room for big MBI elf modules (setup/vmm) */
   . = 0x2000000;
   .stack    : { KEEP(*(.stack*))                 } : phstack

   __kernel_start__ = .;
   .text     : { *(.text*)                        } : phsetup
   .rodata   : { *(.rodata*)                      } : phsetup
   .data     : { *(.data*)                        } : phsetup
   .bss      : { *(.bss* COMMON)                  } : phsetup

   /DISCARD/ : {
        *(.comment*)
        *(.debug*)
        *(.eh_frame*)
        *(.gcc_except_table*)
        *(.note*)
        *(.rel.eh_frame*)
        *(.indent)
   }
}
