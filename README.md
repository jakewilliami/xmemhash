<h1 align="center">xmemhash</h1>

## Description

Extracts a file from a [compressed] archive *in memory* and get its hash.

The inner file is never written to disc for security purposes.

## Quick Start

```bash
$ ./build.sh
$ ./xmemhash -h
```

## Archive Support

Currently, xmemhash supports:
  - Zip
  - 7Zip

Both can optionally be password-protected.
