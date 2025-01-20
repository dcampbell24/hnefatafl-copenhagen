.PHONY: enable-git-hooks
enable-git-hooks:
	git config --local core.hooksPath .githooks/

.PHONY: profile
profile:
	echo '1' | sudo tee /proc/sys/kernel/perf_event_paranoid
	samply record cargo test hnef --profile profiling
