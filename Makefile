# Simple Makefile to build ARSM or run the test cases

TESTS = ./test_cases

.PHONY: main
.DEFAULT: main

main:
	cargo install --path .
