cmake_minimum_required(VERSION 3.10)
project(DxID_Dapp)

set(CMAKE_CXX_STANDARD 17)

# Find CURL
find_package(CURL REQUIRED)
include_directories(${CURL_INCLUDE_DIR})

add_executable(dxid_dapp src/main.cpp)

target_link_libraries(dxid_dapp ${CURL_LIBRARIES})
