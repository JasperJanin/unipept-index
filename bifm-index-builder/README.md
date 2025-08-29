# Bidirectional FM-index builder

A Rust implementation to construct a bidirectional FM-index from a protein database file.
A Postcard binary file is generated, containing the BiFM-index for quick loading.

Program arguments:
- `database`: tsv file containing the protein sequences
- `output_bin`: output file path
- `tsv_field_index`, default 2: index of the tsv field containing the protein sequences
- `sampling_level`, default 3: sampling level of the internal suffix array (1 in 2^(sampling_level) suffixes will be stored)
