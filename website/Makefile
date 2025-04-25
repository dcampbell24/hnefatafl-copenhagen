.PHONY: website
website:
	mdbook build --dest-dir /var/www/html/
	touch /var/www/html/robots.txt
	mkdir --parents /var/www/html/binaries/debian/dists/bookworm/
	cp ../../hnefatafl-copenhagen_0.13.4-1_amd64.deb /var/www/html/binaries/debian/dists/bookworm/
	cp ../packages/Release /var/www/html/binaries/debian/dists/bookworm/
	cp ../packages/Release.gpg /var/www/html/binaries/debian/dists/bookworm/
	mkdir --parents /var/www/html/binaries/nsis/
	cp ../../hnefatafl-client-installer.exe /var/www/html/binaries/nsis/
	cp ../../hnefatafl-client-installer-0.13.4.exe /var/www/html/binaries/nsis/
	sscli -b https://hnefatafl.org -r /var/www/html/
	mkdir /var/www/html/.well-known
	echo "fb1c1fdb-d01d-4918-911f-f4cf4b0540a0" > /var/www/html/.well-known/org.flathub.VerifiedApps.txt

# Install sscli with "npm i -g static-sitemap-cli".

# To update the 404 page:
#     Edit "/etc/apache2/apache.conf"
#     Add the line: "ErrorDocument 404 /404.html"
#     Restart Apache: systemctl restart apache2
