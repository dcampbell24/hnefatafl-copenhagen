#! /bin/sh -e

pandoc\
    --variable=title:hnefatafl-client\
    --variable=section:1\
    --variable=date:2025-02-22\
    --standalone --to=man debian/hnefatafl-client.1.dj --output=debian/hnefatafl-client.1

gzip --no-name --best debian/hnefatafl-client.1

pandoc\
    --variable=title:hnefatafl-server-full\
    --variable=section:1\
    --variable=date:2025-02-22\
    --standalone --to=man debian/hnefatafl-server-full.1.dj --output=debian/hnefatafl-server-full.1

gzip --no-name --best debian/hnefatafl-server-full.1

pandoc --standalone --to=plain README.md --output=debian/README.txt

cargo deb -- --features client,server

rm debian/hnefatafl-client.1.gz
rm debian/hnefatafl-server-full.1.gz
rm debian/README.txt
