-include ../tools.mk

all: $(call NATIVE_STATICLIB,test)
ifeq (,$(filter armv7-unknown-linux-gnueabihf,$(TARGET)))
	$(RUSTC) test.rs
	$(call RUN,test) || exit 1
endif
