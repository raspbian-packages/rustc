# mips64el-unknown-linux-gnuabi64 configuration
CC_mips64el-unknown-linux-gnuabi64=mips64el-linux-gnuabi64-gcc
CXX_mips64el-unknown-linux-gnuabi64=mips64el-linux-gnuabi64-g++
CPP_mips64el-unknown-linux-gnuabi64=mips64el-linux-gnuabi64-gcc
AR_mips64el-unknown-linux-gnuabi64=mips64el-linux-gnuabi64-ar
CFG_LIB_NAME_mips64el-unknown-linux-gnuabi64=lib$(1).so
CFG_STATIC_LIB_NAME_mips64el-unknown-linux-gnuabi64=lib$(1).a
CFG_LIB_GLOB_mips64el-unknown-linux-gnuabi64=lib$(1)-*.so
CFG_LIB_DSYM_GLOB_mips64el-unknown-linux-gnuabi64=lib$(1)-*.dylib.dSYM
CFG_JEMALLOC_CFLAGS_mips64el-unknown-linux-gnuabi64 := -mips64r2 -mabi=64 $(CFLAGS)
CFG_GCCISH_CFLAGS_mips64el-unknown-linux-gnuabi64 := -Wall -g -fPIC -mips64r2 -mabi=64 $(CFLAGS)
CFG_GCCISH_CXXFLAGS_mips64el-unknown-linux-gnuabi64 := -fno-rtti $(CXXFLAGS)
CFG_GCCISH_LINK_FLAGS_mips64el-unknown-linux-gnuabi64 := -shared -fPIC -g -mips64r2
CFG_GCCISH_DEF_FLAG_mips64el-unknown-linux-gnuabi64 := -Wl,--export-dynamic,--dynamic-list=
CFG_LLC_FLAGS_mips64el-unknown-linux-gnuabi64 :=
CFG_INSTALL_NAME_mips64el-unknown-linux-gnuabi64 =
CFG_EXE_SUFFIX_mips64el-unknown-linux-gnuabi64 :=
CFG_WINDOWSY_mips64el-unknown-linux-gnuabi64 :=
CFG_UNIXY_mips64el-unknown-linux-gnuabi64 := 1
CFG_LDPATH_mips64el-unknown-linux-gnuabi64 :=
CFG_RUN_mips64el-unknown-linux-gnuabi64=$(2)
CFG_RUN_TARG_mips64el-unknown-linux-gnuabi64=$(call CFG_RUN_mips64el-unknown-linux-gnuabi64,,$(2))
RUSTC_FLAGS_mips64el-unknown-linux-gnuabi64 :=
CFG_GNU_TRIPLE_mips64el-unknown-linux-gnuabi64 := mips64el-unknown-linux-gnuabi64
