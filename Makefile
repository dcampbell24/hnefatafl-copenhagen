.PHONY: enable-git-hooks
enable-git-hooks:
	git config --local core.hooksPath .githooks/

.PHONY: profile
profile:
	echo '1' | sudo tee /proc/sys/kernel/perf_event_paranoid
	samply record cargo test hnef --profile profiling

.PHONY: logs
logs:
	journalctl --unit=hnefatafl.service --reverse

.PHONY: ssl
ssl:
	openssl \
	req -x509 \
	-newkey rsa:4096 \
	-keyout ssl/localhost.key \
	-out ssl/localhost.crt \
	-sha256 \
	-days 3650 \
	-nodes \
	-subj '/CN=localhost'
	
	sudo cp ssl/localhost.crt /usr/local/share/ca-certificates/
	sudo update-ca-certificates

.PHONY: cargo-deps
cargo-deps:
	python3 ../flatpak-builder-tools/cargo/flatpak-cargo-generator.py Cargo.lock -o cargo-sources.json

.PHONY: js
js:
	wasm-pack build --target web --no-default-features --features js

.PHONY: html-client
html-client:
	sudo cp html-client/index.html /var/www/html/index.html
	sudo mkdir --parent /var/www/html/pkg
	sudo cp -r pkg /var/www/html
