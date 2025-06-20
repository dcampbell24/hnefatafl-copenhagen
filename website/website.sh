#! /bin/bash -ex

mdbook build --dest-dir /var/www/html/

cat << EOF > /var/www/html/robots.txt
User-agent: *
Allow: /

Sitemap: https://hnefatafl.org/sitemap.xml
EOF


mkdir --parents /var/www/html/binaries/debian/
cp ../../hnefatafl-copenhagen_2.1.0-1_amd64.deb /var/www/html/binaries/debian/

mkdir --parents /var/www/html/binaries/nsis/
cp ../../hnefatafl-client-installer.exe /var/www/html/binaries/nsis/
cp ../../hnefatafl-client-installer-0.13.4.exe /var/www/html/binaries/nsis/
cp ../../hnefatafl-client-installer-1.0.0.exe /var/www/html/binaries/nsis/
cp ../../hnefatafl-client-installer-1.1.3.exe /var/www/html/binaries/nsis/
cp ../../hnefatafl-client-installer-1.1.4.exe /var/www/html/binaries/nsis/
cp ../../hnefatafl-client-installer-1.2.1.exe /var/www/html/binaries/nsis/
cp ../../hnefatafl-client-installer-2.0.3.exe /var/www/html/binaries/nsis/
cp ../../hnefatafl-client-installer-2.1.0.exe /var/www/html/binaries/nsis/

sscli -b https://hnefatafl.org -r /var/www/html/

mkdir /var/www/html/.well-known

echo "fb1c1fdb-d01d-4918-911f-f4cf4b0540a0" > /var/www/html/.well-known/org.flathub.VerifiedApps.txt

echo  "514969c804234582abafaae69c947790" > /var/www/html/514969c804234582abafaae69c947790.txt

# Install sscli with "npm i -g static-sitemap-cli".

# To update the 404 page:
#     Edit "/etc/apache2/apache.conf"
#     Add the line: "ErrorDocument 404 /404.html"
#     Restart Apache: systemctl restart apache2
