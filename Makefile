#!/usr/bin/make -f

include config.mk

modules := setup vmm
targets := $(join $(modules), $(addprefix /, $(addsuffix .bin, $(modules))))
objects := $(addsuffix /src/*.o, $(modules))

.PHONY: $(modules) clean

all: $(modules)

$(modules):
	@echo "Building $@"
	@make -C $@

clean:
	@$(RM) -f $(targets) $(objects)

distclean: clean
	@make -C setup $@
	@make -C vmm $@
