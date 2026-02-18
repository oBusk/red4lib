#include <cstdio>
#include <cstdlib>
#include <cstdint>
#include "kraken.h"

int main(int argc, char* argv[]) {
    if (argc != 3) {
        fprintf(stderr, "Usage: %s <input.kark> <output>\n", argv[0]);
        return 1;
    }

    FILE* fin = fopen(argv[1], "rb");
    if (!fin) {
        fprintf(stderr, "Failed to open input file: %s\n", argv[1]);
        return 1;
    }

    // Read .kark header: 4-byte magic "KARK", 4-byte uint32 decompressed size
    uint32_t magic;
    uint32_t decompressed_size;
    if (fread(&magic, sizeof(uint32_t), 1, fin) != 1 ||
        fread(&decompressed_size, sizeof(uint32_t), 1, fin) != 1) {
        fprintf(stderr, "Failed to read header\n");
        fclose(fin);
        return 1;
    }

    if (magic != 0x4B52414B) { // "KRAK"
        fprintf(stderr, "Invalid KARK magic: 0x%08x\n", magic);
        fclose(fin);
        return 1;
    }

    // Read compressed data (everything after the 8-byte header)
    fseek(fin, 0, SEEK_END);
    long file_size = ftell(fin);
    long compressed_size = file_size - 8;
    fseek(fin, 8, SEEK_SET);

    uint8_t* compressed = new uint8_t[compressed_size];
    if (fread(compressed, 1, compressed_size, fin) != (size_t)compressed_size) {
        fprintf(stderr, "Failed to read compressed data\n");
        delete[] compressed;
        fclose(fin);
        return 1;
    }
    fclose(fin);

    // Decompress
    uint8_t* decompressed = new uint8_t[decompressed_size];
    int result = Kraken_Decompress(compressed, compressed_size, decompressed, decompressed_size);
    delete[] compressed;

    if (result != (int)decompressed_size) {
        fprintf(stderr, "Decompression failed: expected %u bytes, got %d\n", decompressed_size, result);
        delete[] decompressed;
        return 1;
    }

    // Write output
    FILE* fout = fopen(argv[2], "wb");
    if (!fout) {
        fprintf(stderr, "Failed to open output file: %s\n", argv[2]);
        delete[] decompressed;
        return 1;
    }

    fwrite(decompressed, 1, decompressed_size, fout);
    fclose(fout);
    delete[] decompressed;

    fprintf(stderr, "Decompressed %ld -> %u bytes\n", compressed_size, decompressed_size);
    return 0;
}
