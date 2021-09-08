cmake_minimum_required(VERSION 3.5)
project(runecoralexample)

set(LIBRUNECORAL_DIR ${CMAKE_SOURCE_DIR}/..)
set(CMAKE_MODULE_PATH "${PROJECT_SOURCE_DIR}/cmake/modules/" ${CMAKE_MODULE_PATH})
include(cmake/EmbedResources.cmake)

find_package(Threads REQUIRED)
set(ADDITIONAL_LIBRARIES "")

if (NOT WIN32)
    # TODO: Make sure to build the correct binaries/libraries on windows too
    find_package(EGL REQUIRED)
    find_package(OpenGLES3 REQUIRED)
    list(APPEND ADDITIONAL_LIBRARIES EGL::EGL OpenGLES3::OpenGLES3)
endif()

if(NOT EXISTS "${CMAKE_BINARY_DIR}/sinemodel.h" OR "sinemodel.tflite" IS_NEWER_THAN "${CMAKE_BINARY_DIR}/sinemodel.h")
    embed_resources("${CMAKE_BINARY_DIR}/sinemodel.h"
                    "sinemodel.tflite")
endif()

add_executable(${PROJECT_NAME} main.cpp)

string(TOLOWER ${CMAKE_SYSTEM_NAME} TARGET_SYSTEM_NAME)

if (NOT CMAKE_CROSSCOMPILING)
    set(LIBRUNECORAL_LIB_DIR "${LIBRUNECORAL_DIR}/dist/lib/${TARGET_SYSTEM_NAME}/x86_64/")
else()
    set(LIBRUNECORAL_LIB_DIR "${LIBRUNECORAL_DIR}/dist/lib/${TARGET_SYSTEM_NAME}/${CMAKE_SYSTEM_PROCESSOR}/")
endif()

target_include_directories(${PROJECT_NAME} PRIVATE ${LIBRUNECORAL_DIR}/runecoral ${CMAKE_BINARY_DIR})
target_link_directories(${PROJECT_NAME} PRIVATE ${LIBRUNECORAL_LIB_DIR})
target_link_libraries(${PROJECT_NAME} PRIVATE
                        runecoral
                        Threads::Threads
                        ${CMAKE_DL_LIBS}
                        ${ADDITIONAL_LIBRARIES})

install(TARGETS ${PROJECT_NAME} DESTINATION ${CMAKE_INSTALL_PREFIX}/)
