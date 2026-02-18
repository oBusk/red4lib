cmake_minimum_required(VERSION 3.10)
project(red4lib_native)

add_subdirectory(kraken EXCLUDE_FROM_ALL)

add_executable(decompress_kark tools/decompress_kark.cpp)
target_link_libraries(decompress_kark PRIVATE kraken_static)
target_include_directories(decompress_kark PRIVATE kraken)

install(TARGETS kraken_static DESTINATION lib)
install(TARGETS decompress_kark DESTINATION bin)
