#!/usr/bin/make -f

include ../config.mk

target   := vmm.bin

rust_src := $(wildcard src/*.rs)
rust_obj := target/x86_64-ros/debug/libvmm.a

asm_src  := $(wildcard src/*.s)
asm_obj  := $(asm_src:.s=.o)

udis_dir := src/disasm/libudis86
udis_src := $(wildcard $(udis_dir)/*.c)
udis_obj := $(udis_src:.c=.o)
udis_lib := $(udis_dir)/libudis86.o

EXTRA_CFLAGS   := $(CFLG_64) -fpie
EXTRA_LDFLAGS  := $(LDFLG_64)
EXTRA2_LDFLAGS := -pie

.PHONY: $(rust_obj)

all: $(target)

$(udis_lib): $(udis_obj)
	@$(aggregate)

# xargo does not support target file from another directory
$(rust_obj): $(rust_src)
	@xargo build --target=x86_64-ros

$(target): $(asm_obj) $(rust_obj) $(udis_lib)
	@$(link)

distclean:
	@xargo clean
