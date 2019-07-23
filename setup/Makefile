#!/usr/bin/make -f

include ../config.mk

target   := setup.bin

rust_src := $(wildcard src/*.rs)
rust_obj := target/x86_64-ros/debug/libsetup.a

asm_src  := $(wildcard src/*.s)
asm_obj  := $(asm_src:.s=.o)

EXTRA_CFLAGS  := $(CFLG_64)
EXTRA_LDFLAGS := $(LDFLG_64)

.PHONY: $(rust_obj)

all: $(target)

# xargo does not support target file from another directory
$(rust_obj): $(rust_src)
	@xargo build --target=x86_64-ros

$(target): $(asm_obj) $(rust_obj)
	@$(link)

distclean:
	@xargo clean
