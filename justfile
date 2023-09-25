set shell := ["cmd.exe", "/c"]

run:
	cargo shuttle run

deploy *ARGS:
	cargo shuttle deploy {{ARGS}}