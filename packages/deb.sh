#! /bin/bash -e

# https://github.com/jgm/pandoc/releases/latest
pandoc\
    --variable=title:hnefatafl-client\
    --variable=section:1\
    --variable=date:2025-02-22\
    --standalone --to=man packages/hnefatafl-client.1.dj --output=packages/hnefatafl-client.1

gzip --no-name --best packages/hnefatafl-client.1

pandoc\
    --variable=title:hnefatafl-server-full\
    --variable=section:1\
    --variable=date:2025-02-22\
    --standalone --to=man packages/hnefatafl-server-full.1.dj --output=packages/hnefatafl-server-full.1

gzip --no-name --best packages/hnefatafl-server-full.1

pandoc --standalone --to=plain README.md --output=packages/README.txt

if [ 'debian' == "$1" ]; then
    PACKAGE=$(cargo deb)
    echo $PACKAGE
    lintian $PACKAGE
fi

rm packages/hnefatafl-client.1.gz
rm packages/hnefatafl-server-full.1.gz
rm packages/README.txt

if [ -z $1 ]; then
    exit
fi

if [ $1 = 'install' ]; then
    sudo dpkg --remove hnefatafl-copenhagen
    sudo dpkg --install $PACKAGE
    sudo systemctl restart hnefatafl.service
    sudo systemctl daemon-reload
fi
