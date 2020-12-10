# Simple Makefile to build ARSM or run the test cases

TESTS = ./test_cases

.PHONY: tests list main
.DEFAULT: main

main:
	cargo install --path .

tests: list

list: $(TESTS)/*.arsm
	@for file in $^ ; do \
		if [ '$(cargo -q run ${file} --stdin $${file/".arsm"/".in"})' != '$(cat ${file/".arsm"/".out"})' ]; then \
			echo "$$(tput bold)$${file}$$(tput sgr0): $$(tput setaf 1)Failed$$(tput sgr0)" ; \
			echo "  * Expected: '$$(cat $${file/".arsm"/".out"})' Got: '$$(cargo -q run $${file} --stdin $${file/".arsm"/".in"})'" ; \
		else \
			echo "$$(tput bold)$${file}$$(tput sgr0): $$(tput setaf 2)Passed$$(tput sgr0)" ; \
		fi \
	done
