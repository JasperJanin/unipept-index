Without alphabet pruning

| Method        | Index size |
|---------------|------------|
| Builtin SA1   | 694MB      |
| Builtin SA32  | 166MB      |
| Builtin SA128 | 1221MB     |

| Method      | Index size | Index build time | S03 count time | S03 retrieve time |
|-------------|------------|------------------|----------------|-------------------|
| Builtin SA1 | 694MB      | 45.0s            | 0.27s          | 37.3s             |
| Builtin SA2 | 430MB      | 45.6s            | 0.28s          | 37.3s             |
| Builtin SA4 | 232MB      | 44.5s            | 0.25s          | 35.5s             |
| Builtin SA8 | 170MB      | 44.7s            | 0.25s          | 35.0s             |

With alphabet pruning

| Method      | Index size | Index build time | S03 count time | S03 retrieve time |
|-------------|------------|------------------|----------------|-------------------|
| Builtin SA1 | 685MB      | 42.9s            | 0.20s          | 29.0s             |
| Builtin SA2 | 421MB      | 42.9s            | 0.21s          | 29.3s             |
| Builtin SA4 | 223MB      | 42.5s            | 0.20s          | 29.0s             |
| Builtin SA8 | 162MB      | 42.5s            | 0.20s          | 29.3s             |

Enkel sequenties, geen metadata 
Vanaf hier: retrieve haalt de positie op, NIET de sequentie

| Method      | Index size | Index build time | S03 count time | S03 retrieve time |
|-------------|------------|------------------|----------------|-------------------|
| Builtin SA1 | 492MB      | 31.9s            | 0.16s          | 0.21s             |
| Builtin SA2 | 311MB      | 31.6s            | 0.16s          | 0.28s             |
| Builtin SA4 | 175MB      | 31.7s            | 0.16s          | 0.78s             |
| Builtin SA8 | 132MB      | 31.6s            | 0.16s          | 12.03s            |

Run-length encoded index met wavelets

| Method              | Index size | Index build time | S03 count time | S03 retrieve time |
|---------------------|------------|------------------|----------------|-------------------|
| Builtin normaal SA1 | 492MB      | 33.0s            | 0.170s         | 0.189s            |
| Builtin wavelet SA1 | 497MB      | 35.9s            | 0.392s         | 0.422s            |
| Builtin normaal SA2 | 311MB      | 32.0s            | 0.163s         | 0.262s            |
| Builtin wavelet SA2 | 316MB      | 34.9s            | 0.377s         | 0.510s            |
| Builtin normaal SA4 | 175MB      | 32.4s            | 0.174s         | 0.745s            |
| Builtin wavelet SA4 | 180MB      | 35.0s            | 0.417s         | 1.09s             |
| Builtin normaal SA6 | 140MB      | 33.8s            | 0.167s         | 2.79s             |
| Builtin wavelet SA6 | 146MB      | 34.1s            | 0.403s         | 3.48s             |
| Builtin normaal SA8 | 132MB      | 33.9s            | 0.160s         | 11.1s             |
| Builtin wavelet SA8 | 137MB      | 34.3s            | 0.400s         | 13.6s             |