cmake_minimum_required(VERSION 3.5)
project(runecoralexample)

set(LIBRUNECORAL_DIR ${CMAKE_SOURCE_DIR}/third_party/librunecoral)
include(cmake/EmbedResources.cmake)

find_package(Threads REQUIRED)

if(NOT EXISTS "${CMAKE_BINARY_DIR}/sinemodel.h" OR "sinemodel.tflite" IS_NEWER_THAN "${CMAKE_BINARY_DIR}/sinemodel.h")
    embed_resources("${CMAKE_BINARY_DIR}/sinemodel.h"
                    "sinemodel.tflite")
endif()

add_executable(${PROJECT_NAME} main.cpp)

string(TOLOWER ${CMAKE_SYSTEM_NAME} TARGET_SYSTEM_NAME)

if (NOT CMAKE_CROSSCOMPILING)
    set(LIBRUNECORAL_LIB_DIR "${LIBRUNECORAL_DIR}/lib/${TARGET_SYSTEM_NAME}/x86_64/")
else()
    set(LIBRUNECORAL_LIB_DIR "${LIBRUNECORAL_DIR}/lib/${TARGET_SYSTEM_NAME}/${CMAKE_SYSTEM_PROCESSOR}/")
endif()

target_include_directories(${PROJECT_NAME} PRIVATE ${LIBRUNECORAL_DIR}/include ${CMAKE_BINARY_DIR})
target_link_libraries(${PROJECT_NAME} PRIVATE -L${LIBRUNECORAL_LIB_DIR} -lrunecoral Threads::Threads ${CMAKE_DL_LIBS})

install(TARGETS ${PROJECT_NAME} DESTINATION ${CMAKE_INSTALL_PREFIX}/)
