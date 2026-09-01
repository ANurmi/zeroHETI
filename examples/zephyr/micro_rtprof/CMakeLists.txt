# SPDX-License-Identifier: Apache-2.0

cmake_minimum_required(VERSION 3.20.0)

find_package(Zephyr REQUIRED HINTS $ENV{ZEPHYR_BASE})
project(micro_rtprof)

if(NOT DEFINED LOAD_FACTOR)
	set(LOAD_FACTOR 50)
endif()
if(NOT DEFINED RUNTIME_MS)
	set(RUNTIME_MS 10)
endif()

set(COMMON_DIR ${CMAKE_CURRENT_SOURCE_DIR}/../common)

target_sources(app PRIVATE src/main.c)

target_include_directories(app PRIVATE src/ ${COMMON_DIR}/)

target_compile_definitions(app PRIVATE
    LOAD_FACTOR=${LOAD_FACTOR}
    RUNTIME_MS=${RUNTIME_MS}
)
