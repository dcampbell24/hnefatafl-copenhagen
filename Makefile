.PHONY: enable-git-hooks
enable-git-hooks:
	git config --local core.hooksPath .githooks/

.PHONY: profile
profile:
	echo '1' | sudo tee /proc/sys/kernel/perf_event_paranoid
	samply record cargo test hnef --profile profiling

.PHONY: ssl
ssl:
	openssl \
	req -x509 \
	-newkey rsa:4096 \
	-keyout testing-ssl/key.pem \
	-out testing-ssl/cert.pem \
	-sha256 \
	-days 3650 \
	-nodes \
	-subj "/C=US/ST=Maine/L=Portland/O=Hnefatafl Org/CN=hnefatafl.org"

.PHONY: website
website:
	mdbook build --dest-dir /var/www/html/
#	sscli -b https://hnefatafl.org -r /var/www/html/
#	touch /var/www/html/robots.txt

# Install sscli with "npm i -g static-sitemap-cli".

# To update the 404 page:
#     Edit "/etc/apache2/apache.conf"
#     Add the line: "ErrorDocument 404 /404.html"
#     Restart Apache: systemctl restart apache2