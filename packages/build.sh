#! /bin/sh -e

if [ 'debian' != "$1" ] && [ 'arch' != "$1" ]; then
    exit 1
fi

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

if [ 'arch' == "$1" ]; then
    cargo build --release --bin hnefatafl-client --features client
    cargo aur
    makepkg --force --dir target/cargo-aur/
    namcap target/cargo-aur/*.zst
fi

rm packages/hnefatafl-client.1.gz
rm packages/hnefatafl-server-full.1.gz
rm packages/README.txt

if [ -z $2 ]; then
    exit
fi

if [ $2 = 'install' ]; then
    sudo dpkg --remove hnefatafl-copenhagen
    sudo dpkg --install $PACKAGE
    sudo systemctl restart hnefatafl.service
    sudo systemctl daemon-reload
fi
