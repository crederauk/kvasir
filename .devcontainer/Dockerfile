# See here for image contents: https://github.com/microsoft/vscode-dev-containers/tree/v0.231.5/containers/rust/.devcontainer/base.Dockerfile

# [Choice] Debian OS version (use bullseye on local arm64/Apple Silicon): buster, bullseye
ARG VARIANT="buster"
FROM mcr.microsoft.com/vscode/devcontainers/rust:0-${VARIANT}

RUN cargo install cargo-edit --features vendored-openssl && \
    cargo install cargo-chef && \
    chown -R vscode /usr/local/cargo/registry/cache/

# [Optional] Uncomment this section to install additional packages.
# RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
#     && apt-get -y install --no-install-recommends <your-package-list-here>
