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

PACKAGE=$(cargo deb)
echo $PACKAGE
lintian $PACKAGE

rm debian/hnefatafl-client.1.gz
rm debian/hnefatafl-server-full.1.gz
rm debian/README.txt

if [ -z $1 ]; then
    exit
fi

if [ $1 = 'install' ]; then
    sudo dpkg --remove hnefatafl-copenhagen
    sudo dpkg --install $PACKAGE
    sudo systemctl enable hnefatafl.service
    sudo systemctl start hnefatafl.service
    sudo systemctl daemon-reload
fi
