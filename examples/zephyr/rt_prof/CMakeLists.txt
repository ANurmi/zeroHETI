# SPDX-License-Identifier: Apache-2.0

cmake_minimum_required(VERSION 3.20.0)

find_package(Zephyr REQUIRED HINTS $ENV{ZEPHYR_BASE})
project(rt_prof)

set(COMMON_DIR ${CMAKE_CURRENT_SOURCE_DIR}/../common)

set(RT_PROF_MAIN "main.c" CACHE STRING "main.c or main_edfic.c")

target_sources(app PRIVATE
    src/${RT_PROF_MAIN}
)

target_include_directories(app PRIVATE
    src/
    ${COMMON_DIR}/
)
