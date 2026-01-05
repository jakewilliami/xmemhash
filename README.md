<h1 align="center">xmemhash</h1>

## Description

Extracts a file from a [compressed] archive *in memory* and get its hash.

The inner file is never written to disc for security purposes.

## Quick Start

```bash
$ just
$ ./xmemhash -h
```

## Archive Support

Currently, xmemhash supports:
  - Zip
  - 7Zip

Both can optionally be password-protected.

## Similar Projects

I have written a sister package to `xmemhash` called [`crlfhash`](https://github.com/jakewilliami/crlfhash).  `crlfhash` will calculate (in-memory) the hashes of a file with different line endings (e.g., with and without a carriage return).

## Citation

If your research depends on `xmemhash`, please consider giving us a formal citation: [`citation.bib`](./citation.bib).
