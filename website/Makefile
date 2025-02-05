.PHONY: website
website:
	mdbook build --dest-dir /var/www/html/
	touch /var/www/html/robots.txt
	sscli -b https://hnefatafl.org -r /var/www/html/

# Install sscli with "npm i -g static-sitemap-cli".

# To update the 404 page:
#     Edit "/etc/apache2/apache.conf"
#     Add the line: "ErrorDocument 404 /404.html"
#     Restart Apache: systemctl restart apache2
