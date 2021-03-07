FROM ubuntu:latest

RUN apt-get update
RUN apt-get -y install clang-11
RUN apt-get -y install llvm-11
RUN apt-get -y install lld-11
RUN apt-get -y install yasm
RUN ln -s /usr/bin/clang-11 /usr/bin/cc
RUN ln -s /usr/bin/clang-11 /usr/bin/gcc
RUN ln -sf /usr/bin/llvm-objcopy-11 /usr/bin/objcopy
RUN ln -sf /usr/bin/ld.lld-11 /usr/bin/ld
