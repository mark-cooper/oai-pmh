.PHONY: ci clippy fmt publish test

ci: fmt clippy test

clippy:
	cargo clippy --all-targets -- -D warnings

fmt:
	cargo fmt --check

publish:
	@git diff-index --quiet HEAD -- || { \
		echo "working tree is dirty; commit the version bump first" >&2; \
		exit 1; \
	}
	@VERSION=$$(cargo metadata --no-deps --format-version 1 \
		| jq -r '.packages[] | select(.name=="archive-it-client") | .version') \
		&& echo "tagging v$$VERSION" \
		&& git tag -a "v$$VERSION" -m "v$$VERSION" \
		&& git push origin "v$$VERSION"

test:
	cargo test
