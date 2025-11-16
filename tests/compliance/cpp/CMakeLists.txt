cmake_minimum_required(VERSION 3.14)
project(lnmp_cpp_compliance_tests)

# Set C++ standard
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Find packages
find_package(GTest REQUIRED)
find_package(yaml-cpp REQUIRED)

# Include directories
include_directories(${GTEST_INCLUDE_DIRS})
include_directories(${YAML_CPP_INCLUDE_DIR})

# Add executable
add_executable(compliance_runner
    runner.cpp
    test_runner.cpp
    test_runner.h
)

# Link libraries
target_link_libraries(compliance_runner
    ${GTEST_BOTH_LIBRARIES}
    yaml-cpp
    pthread
)

# Enable testing
enable_testing()
add_test(NAME ComplianceTests COMMAND compliance_runner)
