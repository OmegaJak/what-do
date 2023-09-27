set shell := ["cmd.exe", "/c"]

watch:
	cargo watch -- just run

run:
	cargo shuttle run

deploy *ARGS:
	cargo shuttle deploy {{ARGS}}