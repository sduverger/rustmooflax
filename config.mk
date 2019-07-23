#!/usr/bin/make -f

MAKEFLAGS  := --no-print-directory
CC_MODEL   := small
CCVER      := 8
CC         := $(shell which gcc-$(CCVER))
CPP        := $(shell which cpp-$(CCVER))
LD         := $(shell which ld)
RM         := $(shell which rm)

CFLG_WRN   := -Wall -W
CFLG_KRN   := -pipe -nostdlib -nostdinc -ffreestanding -fms-extensions
CFLG_FP    := -mno-mmx -mno-sse -mno-sse2 -mno-sse3 -mno-ssse3 -mno-sse4.1 \
              -mno-sse4.2 -mno-sse4 -mno-avx -mno-avx2 -mno-aes -mno-pclmul \
              -mno-fsgsbase -mno-rdrnd -mno-f16c -mno-fma -mno-sse4a \
              -mno-fma4 -mno-xop -mno-lwp -mno-3dnow -mno-popcnt \
              -mno-abm -mno-bmi -mno-bmi2 -mno-lzcnt -mno-tbm

CFLG_64    := -m64 -mno-red-zone -mcmodel=$(CC_MODEL) -D__X86_64__ $(CFLG_FP)
CCLIB_64   := $(shell $(CC) -m64 -print-libgcc-file-name)
LDFLG_64   := -melf_x86_64

CFLAGS     := $(CFLG_WRN) $(CFLG_KRN)
LDFLAGS    := --warn-common --no-check-sections -n
LDSCRIPT   := target.lds

define compile
echo "    CC    $<"
$(CC) $(INCLUDE) $(CFLAGS) $(EXTRA_CFLAGS) -o $@ -c $<
endef

define assemble
echo "    AS    $<"
$(CPP) $< $(CFLAGS) $(EXTRA_CFLAGS) -o $<.s
$(CC) $(CFLAGS) $(EXTRA_CFLAGS) -o $@ -c $<.s 
$(RM) $<.s
endef

define aggregate
echo "    LD    $@"
$(LD) $(LDFLAGS) $(EXTRA_LDFLAGS) -r -o $@ $^
endef

define link
echo "    LD    $@"
$(LD) $(LDFLAGS) $(EXTRA_LDFLAGS) $(EXTRA2_LDFLAGS) --gc-sections -T $(LDSCRIPT) $^ -o $@
endef

%.d: %.c
	@$(depend)
%.d: %.s
	@$(depend)
%.o: %.c
	@$(compile)
%.o: %.s
	@$(assemble)
