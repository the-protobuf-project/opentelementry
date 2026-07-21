cmake_minimum_required(VERSION 3.16)
project(opentelementry-example VERSION 1.0.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Option to use local opentelementry-cpp or fetch from remote
option(USE_LOCAL_OPENTELEMENTRY "Use local opentelementry-cpp directory" ON)

if(USE_LOCAL_OPENTELEMENTRY)
    # Add local opentelementry-cpp as subdirectory
    add_subdirectory(${CMAKE_CURRENT_SOURCE_DIR}/../opentelementry-cpp ${CMAKE_BINARY_DIR}/opentelementry-cpp)
else()
    # Fetch from git (for CI/CD or when not developing locally)
    include(FetchContent)
    FetchContent_Declare(
        opentelementry-cpp
        GIT_REPOSITORY https://github.com/the-protobuf-project/opentelementry.git
        GIT_TAG main
        SOURCE_SUBDIR opentelementry-cpp
    )
    FetchContent_MakeAvailable(opentelementry-cpp)
endif()

add_executable(chat-service main.cpp)
target_link_libraries(chat-service PRIVATE opentelementry)
