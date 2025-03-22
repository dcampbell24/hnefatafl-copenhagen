.PHONY: website
website:
	mdbook build --dest-dir /var/www/html/
	touch /var/www/html/robots.txt
	mkdir --parents /var/www/html/binaries/debian/
	cp ../../hnefatafl-copenhagen_0.12.0-1_amd64.deb /var/www/html/binaries/debian/
	sscli -b https://hnefatafl.org -r /var/www/html/

# Install sscli with "npm i -g static-sitemap-cli".

# To update the 404 page:
#     Edit "/etc/apache2/apache.conf"
#     Add the line: "ErrorDocument 404 /404.html"
#     Restart Apache: systemctl restart apache2
