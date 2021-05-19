cmake_minimum_required(VERSION 3.19)
project(clox C)

set(CMAKE_C_STANDARD 11)

FILE(GLOB SRCS *.c)
add_executable(clox ${SRCS} object.c object.h)