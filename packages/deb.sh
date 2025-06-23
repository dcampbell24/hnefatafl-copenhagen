#! /bin/bash -e

cargo run --bin hnefatafl-ai -- --man --username "" --password "" --role "attacker"
cargo run --example hnefatafl-client -- --man
cargo run -- --man

gzip --no-name --best hnefatafl-ai.1
gzip --no-name --best hnefatafl-client.1
gzip --no-name --best hnefatafl-server-full.1

PACKAGE=$(cargo deb)
echo $PACKAGE
lintian $PACKAGE

rm hnefatafl-ai.1.gz
rm hnefatafl-client.1.gz
rm hnefatafl-server-full.1.gz

if [ -z $1 ]; then
    exit
fi

if [ $1 = 'install' ]; then
    sudo dpkg --remove hnefatafl-copenhagen
    sudo dpkg --install $PACKAGE
    sudo systemctl restart hnefatafl.service
    sudo systemctl daemon-reload
fi
