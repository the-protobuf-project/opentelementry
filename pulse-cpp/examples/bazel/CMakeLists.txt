cmake_minimum_required(VERSION 3.16)
project(pulse-example VERSION 1.0.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Option to use local pulse-cpp or fetch from remote
option(USE_LOCAL_PULSE "Use local pulse-cpp directory" ON)

if(USE_LOCAL_PULSE)
    # Add local pulse-cpp as subdirectory
    add_subdirectory(${CMAKE_CURRENT_SOURCE_DIR}/../pulse-cpp ${CMAKE_BINARY_DIR}/pulse-cpp)
else()
    # Fetch from git (for CI/CD or when not developing locally)
    include(FetchContent)
    FetchContent_Declare(
        pulse-cpp
        GIT_REPOSITORY https://github.com/machanirobotics/pulse.git
        GIT_TAG main
        SOURCE_SUBDIR pulse-cpp
    )
    FetchContent_MakeAvailable(pulse-cpp)
endif()

add_executable(chat-service main.cpp)
target_link_libraries(chat-service PRIVATE pulse)
