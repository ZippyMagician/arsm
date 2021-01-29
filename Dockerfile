# Portable installation of arsm that does not require any dependencies besides Docker itself

# How to use:
#  - Install docker
#  - Run "docker build -t arsm ."
#  - Use the command "docker run --rm arsm {ARGS}" to execute arsm

# Rust base image
FROM clux/muslrust:1.49.0-stable as builder

# Configure inline Python (set to 1 to enable)
ENV INLINE_PYTHON ""

# Get source code
RUN apt update && \
    apt install -y git && \
    git clone https://github.com/ZippyMagician/arsm /home/arsm
WORKDIR /home/arsm

# Build the project
RUN if [ -z ${INLINE_PYTHON} ]; then \
        cargo build --features literal-code --release; \
    else \
        cargo build --features "literal-code inline-python" --release; \
    fi

# Lightweight Linux image
FROM alpine

COPY --from=builder /home/arsm/target/x86_64-unknown-linux-musl/release/arsm .

# Entrypoint is the command
ENTRYPOINT [ "/arsm" ]