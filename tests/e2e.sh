#!/usr/bin/env bash

# End-to-end test suite, testing the functionality of the entire programme
# for different input types (zip and 7z)
#
# TODO: Does not yet test input files with more complex file heirarchy
# TODO: Add Rust unit tests
#
# Run via:
#   $ ./build.sh && ./tests/e2e.sh || rm test-xmemhash.*

# set -xe
trap 'exit 1' INT

FILE="test-xmemhash.txt"
HASH="${1:-sha256}"

if [ -e "$FILE" ]; then
    echo "Cannot test on '$FILE' as it already exists"
    exit 1
fi

echo -n "xmemhash" > "$FILE"

FILE_BASE="$(basename -- $FILE)"
FILE_ZIP="${FILE}.zip"
FILE_ZIP_P="${FILE%.*}.pass.${FILE_BASE##*.}.zip"
FILE_7Z="${FILE}.7z"
FILE_7Z_P="${FILE%.*}.pass.${FILE_BASE##*.}.7z"

zip "$FILE_ZIP" "$FILE" > /dev/null
zip -P infected "$FILE_ZIP_P" "$FILE" > /dev/null
7z a "$FILE_7Z" "$FILE" > /dev/null
7z a -pinfected "$FILE_7Z_P" "$FILE" > /dev/null

echo -n "MD5:    "; md5sum "$FILE"
echo -n "SHA1:   "; sha1sum "$FILE"
echo -n "SHA256: "; sha256sum "$FILE"

./xmemhash --hash "$HASH" "$FILE_ZIP"
./xmemhash --hash "$HASH" "$FILE_ZIP_P"
./xmemhash --hash "$HASH" "$FILE_7Z"
./xmemhash --hash "$HASH" "$FILE_7Z_P"

rm "$FILE"
rm "$FILE_ZIP"
rm "$FILE_ZIP_P"
rm "$FILE_7Z"
rm "$FILE_7Z_P"
