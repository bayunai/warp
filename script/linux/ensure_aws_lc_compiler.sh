#!/usr/bin/env bash
#
# aws-lc-sys rejects GCC 9 (Ubuntu 20.04 default) due to a memcmp codegen bug
# (https://gcc.gnu.org/bugzilla/show_bug.cgi?id=95189). Prefer GCC 10+ when
# installed and CC/CXX are unset.

if [[ -z "${CC:-}" && -z "${CXX:-}" ]] && command -v gcc-10 >/dev/null 2>&1 && command -v g++-10 >/dev/null 2>&1; then
  export CC=gcc-10
  export CXX=g++-10
fi
