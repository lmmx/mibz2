install-hooks:
   pre-commit install

run-pc:
   pre-commit run --all-files

# -------------------------------------

fix-eof-ws mode="":
    #!/usr/bin/env sh
    ARGS=''
    if [ "{{mode}}" = "check" ]; then
        ARGS="--check-only"
    fi
    whitespace-format --add-new-line-marker-at-end-of-file \
          --new-line-marker=linux \
          --normalize-new-line-markers \
          --exclude ".git/|target/|dist/|site/|__pycache__/|\.pyc$|\.swp|.egg-info/|\.so$|.json$|.lock$|.parquet$|\.gz$|\.xz$|.venv/|\..*cache/" \
          $ARGS \
          .

code-quality:
    # just ty-ci
    taplo lint
    taplo format --check
    just fix-eof-ws check
    just _lint-cargo machete
    just _lint-cargo fmt --check --all

# Lint-only cargo commands (machete, fmt) under the pinned toolchain when it is installed,
# else under any installed one: pre-commit.ci runs offline and cannot fetch the pin.
# Nothing here compiles, so the version doesn't change the result.
_lint-cargo *args:
    #!/usr/bin/env sh
    pin="$(sed -n 's/^channel *= *"\(.*\)"/\1/p' rust-toolchain.toml)"
    if ! RUSTUP_AUTO_INSTALL=0 rustup run "$pin" rustc -V >/dev/null 2>&1; then
        RUSTUP_TOOLCHAIN="$(rustup toolchain list | head -n1 | cut -d' ' -f1)"
        export RUSTUP_TOOLCHAIN
        echo "lint: the pinned $pin isn't installed here; using $RUSTUP_TOOLCHAIN" >&2
    fi
    cargo {{args}}

code-quality-fix:
    taplo lint
    taplo format
    just fix-eof-ws
    cargo machete
    cargo fmt --all
